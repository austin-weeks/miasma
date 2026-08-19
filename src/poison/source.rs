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

    /// This _technically_ only inserts into the cache if the stream is fully drained
    /// by the scraper, which should almost always be the case.
    async fn fetch_then_insert(&self, cache: Arc<Mutex<PoisonCache>>) -> Option<impl MiasmaStream> {
        // Poison responses are typically 4-8 KB
        const INITIAL_BUF_CAPACITY: usize = 4 * 1024; // 4 KB

        let stream = self.client.stream_poison().await?;

        Some(try_stream! {
            let mut buf = BytesMut::with_capacity(INITIAL_BUF_CAPACITY);
            let mut stream = pin!(stream);
            while let Some(chunk) = stream.next().await {
                let bytes = chunk?;
                buf.extend_from_slice(&bytes);
                yield bytes;
            }
            let entry_size = buf.capacity();
            let mut cache_guard = cache.lock().expect("cache mutex poisoned");
            if cache_guard.insert(buf.freeze(), Some(entry_size)).is_err() {
                eprintln!(
                    "Poison too big to store in poison cache - exceeds max cache byte size of {}",
                    cache_guard.max_bytes(),
                );
            }
        })
    }

    fn cache_entry(cache: &Arc<Mutex<PoisonCache>>) -> Option<impl MiasmaStream> {
        let entry = cache.lock().expect("cache mutex poisoned").get_random()?;
        Some(stream::once(async { Ok(entry) }))
    }
}

#[cfg(test)]
mod test {
    use axum::{Router, routing::get};
    use url::Url;

    use crate::test_utils::{self, TestServer};

    use super::*;

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
            assert!(!resp.is_empty());
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
            assert!(!resp.is_empty());
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
            assert!(!resp.is_empty());
        }
    }
}
