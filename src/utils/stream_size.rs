use std::pin::pin;

use async_stream::try_stream;
use futures::StreamExt;

use crate::MiasmaStream;

/// Calls the provided function with the total byte size of the stream upon completion.
/// Returns the original stream unaltered.
pub fn with_bytes_counted(
    stream: impl MiasmaStream,
    record_size: impl AsyncFnOnce(usize),
) -> impl MiasmaStream {
    let mut stream_size = 0;

    try_stream! {
        let mut stream = pin!(stream);
        while let Some(chunk) = stream.next().await {
            if let Ok(ref chunk) = chunk {
                stream_size += chunk.len();
            }
            yield chunk?;
        }
        record_size(stream_size).await;
    }
}

#[cfg(test)]
mod test {
    use async_stream::try_stream;

    use crate::test_utils;

    use super::*;

    #[tokio::test]
    async fn counts_bytes_and_does_not_mutate() {
        let stream = try_stream! {
            yield "hello".into();
            yield " ".into();
            yield "world".into();
        };

        let mut bytes = 0;
        let with_size = with_bytes_counted(stream, async |n| bytes = n);

        let result = test_utils::drain_byte_stream(with_size).await;
        #[expect(clippy::needless_as_bytes)]
        let expected_size = "hello world".as_bytes().len();

        assert_eq!(result, "hello world");
        assert_eq!(bytes, expected_size);
    }
}
