// austin-weeks: I've left lots of comments in this file as I've made many
// decisions that I'd like to keep transparent for the next person who looks
// at this file. This is a rare case of "the code does not document itself".
use std::{
    pin::Pin,
    task::{Context, Poll},
};

use bytes::Bytes;
use futures::{Stream, StreamExt};
use thiserror::Error;
use zlib_rs::{Deflate, DeflateError, DeflateFlush, Status};

use crate::{MiasmaStream, gzip::GzipSettings, ternary};

const OUTPUT_BUFFER_SIZE: usize = 512;

/// Gzip-compressed stream tweaked for ultra-low memory utilization.
/// Compared to the `async-compression` crate's `GzipEncoder`, this uses
/// _vastly_ less memory for only slightly worse compression ratios.
pub struct LowMemGzipStream<E> {
    state: CompressState,
    stream: Pin<Box<dyn MiasmaStream<E> + Send>>,
    input_buf: Bytes,
    input_cursor: usize,
    deflate: Deflate,
}

enum CompressState {
    Pull,
    Drain,
    Finish,
    Done,
}

struct CompressedChunk {
    chunk: Option<Bytes>,
    stream_done: bool,
}

#[derive(Error, Debug)]
pub enum CompressError<E> {
    #[error(transparent)]
    Stream(E),
    #[error("gzip compression failed to make progress - invariants violated")]
    Buffer,
    #[error("failed to compress stream: {0:?}")]
    Deflate(DeflateError),
}

impl<E> LowMemGzipStream<E> {
    pub fn new(stream: impl MiasmaStream<E> + 'static + Send) -> Self {
        // TODO: make this an input and pipe through app starting at config / cli.
        let config = GzipSettings::default().into_deflate_config();

        Self {
            state: CompressState::Pull,
            stream: Box::pin(stream),
            deflate: Deflate::new_with_config(config),
            input_buf: Bytes::new(),
            input_cursor: 0,
        }
    }

    fn input_consumed(&self) -> bool {
        self.input_cursor >= self.input_buf.len()
    }

    fn compress(&mut self) -> Result<CompressedChunk, CompressError<E>> {
        // FIXME: we _maybe_ could further optimize this by using a Arc<Vec<u8>>
        // to share the buffer between yields, but that is only a win if we can prove
        // that hyper/axum fully sends the Bytes chunk _before_ pulling the next.
        // At least try it, benchmark, and document that it's a bad approach if so.

        // 1/2 KB buffer. Sets the vec's length to equal capacity. This is
        // necessary as Deflate uses length to determine size of the buffer.
        //
        // I'm opting to create the buffer inline rather than as a reused field
        // on the struct as the returned Bytes needs to take ownership of the
        // buffer. This would use more memory overall as we'd need:
        // (struct's buffer) + (in-flight buffer) * (num-in-flight chunks), whereas
        // inline just needs: (in-flight buffer) * (num-in-flight chunks).
        // If there were a way to reuse a buffer in-between chunks without copying,
        // that would be more efficient, but because Bytes takes ownership this is
        // not the case.
        let mut output_buf = vec![0u8; OUTPUT_BUFFER_SIZE];

        // Deflate tracks the total bytes in/out so we have to save the prior values.
        let prior_out = self.deflate.total_out();
        let prior_in = self.deflate.total_in();

        // Grab the un-processed slice of the current stream chunk.
        let input = &self.input_buf[self.input_cursor..];

        let stream_done = match self.deflate.compress(
            input,
            &mut output_buf,
            ternary!(
                matches!(self.state, CompressState::Finish),
                DeflateFlush::Finish,
                DeflateFlush::NoFlush
            ),
        ) {
            Ok(Status::Ok) => false,
            Ok(Status::StreamEnd) => true,
            Ok(Status::BufError) => return Err(CompressError::Buffer),
            Err(e) => return Err(CompressError::Deflate(e)),
        };

        let now_out = to_usize(self.deflate.total_out() - prior_out);
        let now_in = to_usize(self.deflate.total_in() - prior_in);

        // We should never hit this (zlib should return BufError), but if
        // it makes literally 0 progress, we don't want to loop forever.
        assert!(now_in > 0 || now_out > 0);

        // Advance the input buffer cursor forward by amount encoded.
        self.input_cursor += now_in;

        // Set the output buffer's length to wherever was written to.
        output_buf.truncate(now_out);

        // This occurs if zlib consumes input but decides not to emit
        // output bytes. This happens often and is not an error state.
        // In this case, emit nothing and don't send upstream.
        if output_buf.is_empty() {
            return Ok(CompressedChunk {
                chunk: None,
                stream_done,
            });
        }

        Ok(CompressedChunk {
            chunk: Some(Bytes::from(output_buf)),
            stream_done,
        })
    }
}

impl<E> Stream for LowMemGzipStream<E> {
    type Item = Result<Bytes, CompressError<E>>;

    fn poll_next(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        loop {
            match self.state {
                CompressState::Pull => {
                    let chunk = match self.stream.poll_next_unpin(cx) {
                        Poll::Pending => return Poll::Pending,
                        Poll::Ready(None) => {
                            self.state = CompressState::Finish;
                            continue;
                        }
                        Poll::Ready(Some(Err(e))) => {
                            return Poll::Ready(Some(Err(CompressError::Stream(e))));
                        }
                        Poll::Ready(Some(Ok(c))) => c,
                    };

                    // Cannot compress an empty buffer. Pull the next chunk.
                    if chunk.is_empty() {
                        continue;
                    }

                    self.input_buf = chunk;
                    self.input_cursor = 0;
                    self.state = CompressState::Drain;
                }
                CompressState::Drain => {
                    let CompressedChunk { chunk, .. } = match self.compress() {
                        Ok(c) => c,
                        Err(e) => return Poll::Ready(Some(Err(e))),
                    };
                    // If we have no input left to encode, pull the next chunk.
                    if self.input_consumed() {
                        self.state = CompressState::Pull;
                    }
                    // Valid for compress to return no chunk, just continue to next iteration.
                    let Some(chunk) = chunk else {
                        continue;
                    };
                    return Poll::Ready(Some(Ok(chunk)));
                }
                CompressState::Finish => {
                    // If the final input buffer has been consumed, replace with an empty buffer
                    // to ensure we don't re-encode data.
                    if self.input_consumed() {
                        self.input_cursor = 0;
                        self.input_buf = Bytes::new();
                    }
                    let CompressedChunk { stream_done, chunk } = match self.compress() {
                        Ok(c) => c,
                        Err(e) => return Poll::Ready(Some(Err(e))),
                    };
                    if stream_done {
                        self.state = CompressState::Done;
                    }
                    // Valid for compress to return no chunk, just continue to next iteration.
                    let Some(chunk) = chunk else {
                        continue;
                    };
                    return Poll::Ready(Some(Ok(chunk)));
                }
                CompressState::Done => return Poll::Ready(None),
            }
        }
    }
}

fn to_usize(size: u64) -> usize {
    size.try_into()
        // Otherwise that means we tried to encode > 4,294,967,295 bytes on a 32-bit machine...
        .expect("size should always be less than u32::MAX")
}

#[cfg(test)]
mod test {
    use async_compression::tokio::bufread::GzipDecoder;
    use async_stream::try_stream;
    use futures::{TryStreamExt, io};
    use rand::random_range;
    use tokio::io::BufReader;
    use tokio_util::io::{ReaderStream, StreamReader};

    use crate::test_utils;

    use super::*;

    /// Creates a stream of [0, 100] random integers.
    fn create_test_stream() -> (impl MiasmaStream, String) {
        let data = (0..=random_range(0..=100))
            .map(|_| random_range(i32::MIN..=i32::MAX).to_string())
            .collect::<Vec<_>>();
        let expected = data.iter().map(String::as_str).collect::<String>();
        let stream = try_stream! {
            for num_str in data {
                yield Bytes::from(num_str);
            }
        };
        (stream, expected)
    }

    async fn decode_and_drain<E>(gzip_stream: LowMemGzipStream<E>) -> String
    where
        E: std::error::Error + Send + Sync + 'static,
    {
        let reader = StreamReader::new(gzip_stream.map_err(io::Error::other));
        let buf_reader = BufReader::new(reader);
        let decoder = GzipDecoder::new(buf_reader);
        let stream = ReaderStream::new(decoder);

        test_utils::drain_byte_stream(stream).await
    }

    #[tokio::test]
    async fn produces_valid_gzipped_data() {
        const TEST_ITERATIONS: usize = 100;
        for _ in 0..TEST_ITERATIONS {
            let (stream, expected) = create_test_stream();
            let gzip_stream = LowMemGzipStream::new(stream);

            let result = decode_and_drain(gzip_stream).await;

            assert_eq!(
                result, expected,
                "decoded stream does not match expected input data"
            );
        }
    }

    // We could go crazy with unit tests, but most of that functionality is covered by the above
    // round trip test. I've included a couple tests excercising bugs that I had to work through.

    #[tokio::test]
    /// zlib will return a `BufError` if we try to encode an empty input buffer.
    async fn skips_empty_stream_chunks() {
        let stream = try_stream! {
            yield Bytes::from("a");
            yield Bytes::new();
            yield Bytes::from("b");
            yield Bytes::new();
            yield Bytes::from("c");
            yield Bytes::new();
        };
        let expected = "abc";
        let gzip_stream = LowMemGzipStream::<io::Error>::new(stream);

        let result = decode_and_drain(gzip_stream).await;

        assert_eq!(result, expected);
    }

    #[test]
    fn correctly_advances_input_cursor() {
        let stream = try_stream! {
            yield Bytes::new();
        };
        let mut gzip_stream = LowMemGzipStream::<io::Error>::new(stream);

        gzip_stream.input_cursor = 0;
        gzip_stream.input_buf = Bytes::from(vec![0u8; 3]);
        gzip_stream.compress().unwrap();
        assert_eq!(gzip_stream.input_cursor, 3);

        gzip_stream.input_cursor = 0;
        gzip_stream.input_buf = Bytes::from(vec![0u8; 5]);
        gzip_stream.compress().unwrap();
        // Cursor should NOT advance by the total number of bytes that deflate has consumed.
        assert_ne!(gzip_stream.input_cursor, 8);
        assert_eq!(gzip_stream.input_cursor, 5);
    }
}
