use std::{
    pin::pin,
    sync::{Arc, Mutex},
};

use async_stream::try_stream;
use bytes::BytesMut;
use futures::{StreamExt, stream};

use crate::{
    MiasmaStream,
    poison::{FallbackMiasmaStreamExt, HtmlEscapeMode, PoisonClient, cache::PoisonCache},
    ternary,
};

pub struct PoisonSource {
    client: PoisonClient,
    fallback_escape_mode: HtmlEscapeMode,
    cache: Option<SourceCache>,
}

struct SourceCache {
    cache: Arc<Mutex<PoisonCache>>,
    cache_hit_fn: Box<dyn Fn() -> bool + Send + Sync>,
}

impl PoisonSource {
    pub fn new(client: PoisonClient, fallback_poison_escape_mode: HtmlEscapeMode) -> Self {
        Self {
            client,
            cache: None,
            fallback_escape_mode: fallback_poison_escape_mode,
        }
    }

    /// The intended `cache_hit_fn` is produced by [`crate::poison::cache::make_cache_hit_fn`].
    pub fn with_cache(
        mut self,
        cache: PoisonCache,
        cache_hit_fn: Box<dyn Fn() -> bool + Send + Sync>,
    ) -> Self {
        self.cache = Some(SourceCache {
            cache_hit_fn,
            cache: Arc::new(Mutex::new(cache)),
        });
        self
    }

    pub async fn stream_poison(&self) -> impl MiasmaStream {
        let Some(SourceCache {
            cache,
            cache_hit_fn,
        }) = &self.cache
        else {
            return self
                .client
                .stream_poison()
                .await
                .or_fallback(self.fallback_escape_mode)
                .boxed();
        };

        ternary!(
            cache_hit_fn(),
            match Self::cache_entry(cache) {
                Some(stream) => stream.boxed(),
                None => self
                    .fetch_then_insert(Arc::clone(cache))
                    .await
                    .or_fallback(self.fallback_escape_mode)
                    .boxed(),
            },
            match self.fetch_then_insert(Arc::clone(cache)).await {
                Some(stream) => stream.boxed(),
                None => Self::cache_entry(cache)
                    .or_fallback(self.fallback_escape_mode)
                    .boxed(),
            },
        )
    }

    /// Cache the response only after the stream is fully drained.
    async fn fetch_then_insert(&self, cache: Arc<Mutex<PoisonCache>>) -> Option<impl MiasmaStream> {
        // Poison responses are typically 4-8 KB
        const INITIAL_BUF_CAPACITY: usize = 4 * 1024; // 4 KB

        let stream = self.client.stream_poison().await?;
        let max_bytes = cache.lock().expect("cache mutex poisoned").max_bytes();

        Some(try_stream! {
            let mut buf = Some(BytesMut::with_capacity(INITIAL_BUF_CAPACITY.min(max_bytes)));
            let mut stream = pin!(stream);
            while let Some(chunk) = stream.next().await {
                let bytes = chunk?;
                extend_cache_buffer(&mut buf, &bytes, max_bytes);
                yield bytes;
            }
            if let Some(buf) = buf {
                let entry_size = buf.capacity();
                let mut cache_guard = cache.lock().expect("cache mutex poisoned");
                let _ = cache_guard.insert(buf.freeze(), Some(entry_size));
            }
        })
    }

    fn cache_entry(cache: &Arc<Mutex<PoisonCache>>) -> Option<impl MiasmaStream> {
        let entry = cache.lock().expect("cache mutex poisoned").get_random()?;
        Some(stream::once(async { Ok(entry) }))
    }
}

fn extend_cache_buffer(buffer: &mut Option<BytesMut>, bytes: &[u8], max_bytes: usize) {
    let Some(buf) = buffer else { return };
    if bytes.len() > max_bytes.saturating_sub(buf.len()) {
        *buffer = None;
        return;
    }
    let needed = buf.len() + bytes.len();
    if needed > buf.capacity() {
        // BytesMut's automatic growth may exceed the cache budget.
        let capacity = needed.max(buf.capacity().saturating_mul(2)).min(max_bytes);
        let mut grown = BytesMut::with_capacity(capacity);
        grown.extend_from_slice(buf);
        *buf = grown;
    }
    buf.extend_from_slice(bytes);
}

#[cfg(test)]
mod test {
    use axum::{Router, routing::get};
    use url::Url;

    use crate::test_utils::{self, TestServer};

    use super::*;

    #[test]
    fn cache_buffer_stays_bounded_and_stops_after_overflow() {
        let mut buffer = Some(BytesMut::with_capacity(4));
        for bytes in [b"1234".as_slice(), b"5", b"6"] {
            extend_cache_buffer(&mut buffer, bytes, 6);
            let buf = buffer.as_ref().unwrap();
            assert!(buf.capacity() <= 6);
        }
        assert_eq!(buffer.as_ref().unwrap().as_ref(), b"123456");
        extend_cache_buffer(&mut buffer, b"7", 6);
        assert!(buffer.is_none());
        extend_cache_buffer(&mut buffer, b"", 6);
        assert!(buffer.is_none());
    }

    #[test]
    fn cache_buffer_rejects_oversized_first_chunk() {
        let mut buffer = Some(BytesMut::new());
        extend_cache_buffer(&mut buffer, b"1234567", 6);
        assert!(buffer.is_none());
    }

    #[tokio::test]
    async fn oversized_poison_is_streamed_without_evicting_cached_data() {
        let cache = PoisonCache::new();
        let size = cache.max_bytes() + 1;
        let (client, _server) = client_and_server_with(&"x".repeat(size)).await;
        let source =
            PoisonSource::new(client, HtmlEscapeMode::Escape).with_cache(cache, Box::new(|| false));
        let cache = &source.cache.as_ref().unwrap().cache;
        cache
            .lock()
            .unwrap()
            .insert(bytes::Bytes::from_static(b"old"), None)
            .unwrap();
        let mut poison = pin!(source.stream_poison().await);
        let mut received = 0;
        while let Some(chunk) = poison.next().await {
            received += chunk.unwrap().len();
        }
        assert_eq!(received, size);
        assert_eq!(cache.lock().unwrap().get_random().unwrap(), "old");
    }

    fn client_that_errors() -> PoisonClient {
        PoisonClient::new(
            Url::parse("http://invalid.").unwrap(),
            HtmlEscapeMode::Escape,
        )
    }

    async fn client_and_server_with(response: &str) -> (PoisonClient, TestServer) {
        let response = response.to_owned();
        let router = Router::new().fallback(get(async move || response));
        let server = test_utils::test_server(router).await;
        (
            PoisonClient::new(server.url.clone(), HtmlEscapeMode::Escape),
            server,
        )
    }

    #[test]
    fn constructors() {
        let source = PoisonSource::new(client_that_errors(), HtmlEscapeMode::Escape);
        assert!(matches!(
            source.fallback_escape_mode,
            HtmlEscapeMode::Escape
        ));
        assert!(source.cache.is_none());

        let source_w_cache = source.with_cache(PoisonCache::new(), Box::new(|| true));
        assert!(source_w_cache.cache.is_some());
    }

    #[tokio::test]
    async fn client_response_cached() {
        let (client, _) = client_and_server_with("client poison!").await;
        let source = PoisonSource::new(client, HtmlEscapeMode::Escape)
            .with_cache(PoisonCache::new(), Box::new(|| false));
        let resp = test_utils::drain_byte_stream(source.stream_poison().await).await;
        assert_eq!(resp, "client poison!");
        let cached = source
            .cache
            .unwrap()
            .cache
            .lock()
            .unwrap()
            .get_random()
            .unwrap();
        assert_eq!(cached, "client poison!");
    }

    #[tokio::test]
    async fn dropped_stream_preserves_cached_data() {
        let (client, _server) = client_and_server_with("client poison!").await;
        let source = PoisonSource::new(client, HtmlEscapeMode::Escape)
            .with_cache(PoisonCache::new(), Box::new(|| false));
        let cache = &source.cache.as_ref().unwrap().cache;
        cache
            .lock()
            .unwrap()
            .insert(bytes::Bytes::from_static(b"old"), None)
            .unwrap();
        {
            let mut stream = pin!(source.stream_poison().await);
            assert!(!stream.next().await.unwrap().unwrap().is_empty());
        }
        assert_eq!(cache.lock().unwrap().get_random().unwrap(), "old");
    }

    // So many cases to cover D:
    mod stream_poison {
        use bytes::Bytes;

        use super::*;

        // No Cache
        #[tokio::test]
        async fn no_cache_client_success() {
            let (client, _) = client_and_server_with("poison!").await;
            let source = PoisonSource::new(client, HtmlEscapeMode::Escape);
            let resp = test_utils::drain_byte_stream(source.stream_poison().await).await;
            assert_eq!(resp, "poison!");
        }

        #[tokio::test]
        async fn no_cache_client_failure() {
            let source = PoisonSource::new(client_that_errors(), HtmlEscapeMode::Escape);
            let resp = test_utils::drain_byte_stream(source.stream_poison().await).await;
            assert_ne!(resp, "");
        }

        // Cache Hit
        #[tokio::test]
        async fn cache_hit_nonempty_cache() {
            let mut cache = PoisonCache::new();
            cache.insert(Bytes::from("cached poison!"), None).unwrap();
            let source = PoisonSource::new(client_that_errors(), HtmlEscapeMode::Escape)
                .with_cache(cache, Box::new(|| true));
            let resp = test_utils::drain_byte_stream(source.stream_poison().await).await;
            assert_eq!(resp, "cached poison!");
        }

        #[tokio::test]
        async fn cache_hit_empty_cache_client_success() {
            let (client, _) = client_and_server_with("client poison!").await;
            let source = PoisonSource::new(client, HtmlEscapeMode::Escape)
                .with_cache(PoisonCache::new(), Box::new(|| true));
            let resp = test_utils::drain_byte_stream(source.stream_poison().await).await;
            assert_eq!(resp, "client poison!");
        }

        #[tokio::test]
        async fn cache_hit_empty_cache_client_failure() {
            let source = PoisonSource::new(client_that_errors(), HtmlEscapeMode::Escape)
                .with_cache(PoisonCache::new(), Box::new(|| true));
            let resp = test_utils::drain_byte_stream(source.stream_poison().await).await;
            assert_ne!(resp, "");
        }

        // Cache Miss
        #[tokio::test]
        async fn cache_miss_client_success() {
            let (client, _) = client_and_server_with("client poison!").await;
            let mut cache = PoisonCache::new();
            cache.insert(Bytes::from("cached poison!"), None).unwrap();
            let source = PoisonSource::new(client, HtmlEscapeMode::Escape)
                .with_cache(cache, Box::new(|| false));
            let resp = test_utils::drain_byte_stream(source.stream_poison().await).await;
            assert_eq!(resp, "client poison!");
            assert_ne!(resp, "cached poison!");
        }

        #[tokio::test]
        async fn cache_miss_client_failure_nonempty_cache() {
            let mut cache = PoisonCache::new();
            cache.insert(Bytes::from("cached poison!"), None).unwrap();
            let source = PoisonSource::new(client_that_errors(), HtmlEscapeMode::Escape)
                .with_cache(cache, Box::new(|| false));
            let resp = test_utils::drain_byte_stream(source.stream_poison().await).await;
            assert_eq!(resp, "cached poison!");
        }

        #[tokio::test]
        async fn cache_miss_client_failure_empty_cache() {
            let source = PoisonSource::new(client_that_errors(), HtmlEscapeMode::Escape)
                .with_cache(PoisonCache::new(), Box::new(|| false));
            let resp = test_utils::drain_byte_stream(source.stream_poison().await).await;
            assert_ne!(resp, "");
        }
    }
}
