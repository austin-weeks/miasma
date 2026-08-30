use std::io;

use futures::TryStreamExt;
use tokio::io::BufReader;
use tokio_util::io::{ReaderStream, StreamReader};

use crate::MiasmaStream;

const DEFAULT_BUFFER_SIZE: usize = 1024; // 1 KB

/// Buffer the stream into fixed size chunks.
#[allow(unused)]
pub fn buffer_stream<E>(
    stream: impl MiasmaStream<E>,
    buffer_size: Option<usize>,
) -> impl MiasmaStream<io::Error>
where
    E: Into<Box<dyn std::error::Error + Send + Sync>>,
{
    let stream = stream.map_err(io::Error::other);
    let reader = StreamReader::new(stream);
    let buf = BufReader::with_capacity(buffer_size.unwrap_or(DEFAULT_BUFFER_SIZE), reader);
    ReaderStream::with_capacity(buf, buffer_size.unwrap_or(DEFAULT_BUFFER_SIZE))
}
