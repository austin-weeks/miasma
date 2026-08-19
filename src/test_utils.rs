use std::{fmt::Debug, pin::pin};

use axum::Router;
use bytes::Bytes;
use futures::{Stream, StreamExt};
use tempfile::NamedTempFile;
use tokio::{net::TcpListener, task::JoinHandle};
use url::Url;

pub struct TestServer {
    pub url: Url,
    _running_server: JoinHandle<()>,
}

/// Spawn an ephemeral server with the provided router.
///
/// # Panics
/// Panics on failure to bind port or start server task.
pub async fn test_server(app: Router) -> TestServer {
    let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
    TestServer {
        url: format!("http://{}", listener.local_addr().unwrap())
            .parse()
            .unwrap(),
        _running_server: tokio::spawn(async move { axum::serve(listener, app).await.unwrap() }),
    }
}

/// Creates a temporary file.
///
/// # Panics
/// Panics on failure to create temp file or failure to convert file name to `String`.
pub fn temp_file() -> (NamedTempFile, String) {
    let temp_file = NamedTempFile::new().unwrap();
    let path = temp_file.path().to_str().unwrap().to_owned();
    (temp_file, path)
}

/// Drains the provided stream and collects into a `String`.
///
/// # Panics
/// Panics if a yielded `Result` chunk is `Err` or yielded `Bytes` is not a valid UTF-8 string.
pub async fn drain_byte_stream<E>(stream: impl Stream<Item = Result<Bytes, E>>) -> String
where
    E: Debug,
{
    let mut buf = String::new();
    let mut stream = pin!(stream);
    while let Some(chunk) = stream.next().await {
        buf.push_str(str::from_utf8(&chunk.unwrap()).unwrap());
    }
    buf
}
