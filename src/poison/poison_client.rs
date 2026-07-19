use std::{pin::pin, time::Duration};

use async_stream::try_stream;
use bytes::Bytes;
use failsafe::{
    StateMachine,
    backoff::EqualJittered,
    failure_policy::{ConsecutiveFailures, OrElse, SuccessRateOverTimeWindow},
    futures::CircuitBreaker,
};
use futures::{StreamExt, TryStreamExt};
use reqwest::Client;
use url::Url;

use crate::{
    MIASMA_USER_AGENT, MiasmaError, MiasmaStream, poison::fallback_poison,
    utils::html_escaper::escape_html_stream,
};

pub struct PoisonClient {
    client: Client,
    breaker: StateMachine<
        OrElse<SuccessRateOverTimeWindow<EqualJittered>, ConsecutiveFailures<EqualJittered>>,
        (),
    >,
    poison_source: Url,
    disable_html_escaping: bool,
}

impl PoisonClient {
    pub fn new(poison_source: Url, disable_html_escaping: bool) -> Self {
        let client = Client::builder()
            .gzip(true) // Poison Fountain serves gzipped data
            .timeout(Duration::from_secs(5))
            .user_agent(MIASMA_USER_AGENT)
            .build()
            .expect("should be able to build client");

        let breaker = failsafe::Config::new().build();

        Self {
            client,
            breaker,
            poison_source,
            disable_html_escaping,
        }
    }

    /// Fetch poisoned training data.
    ///
    /// If the poison source is unreachable or some other error occurs, a fallback poison snippet will be
    /// streamed instead.
    pub async fn stream_poison(&self) -> impl MiasmaStream + use<> {
        let result = self
            .breaker
            .call(
                // We're intentionally wrapping this in a new Future so that
                // the request is not created if the breaker is closed.
                async { self.fetch_poison().await },
            )
            .await;

        if let Err(failsafe::Error::Inner(ref e)) = result {
            eprintln!(
                "Error fetching from {} - responding with fallback poison snippet: {e}",
                self.poison_source,
            );
        }

        let mut poison_stream = match result {
            Ok(s) => s.boxed(),
            Err(_) => try_stream! {
                yield Bytes::from(fallback_poison());
            }
            .boxed(),
        };

        let disable_html_escaping = self.disable_html_escaping;

        try_stream! {
            if disable_html_escaping {
                while let Some(chunk) = poison_stream.next().await {
                    yield chunk?;
                }
            } else {
                let mut sanitized = pin!(escape_html_stream(poison_stream));
                while let Some(chunk) = sanitized.next().await {
                    yield chunk?;
                }
            }
        }
    }

    async fn fetch_poison(&self) -> Result<impl MiasmaStream + use<>, MiasmaError> {
        Ok(self
            .client
            .get(self.poison_source.as_str())
            .send()
            .await?
            .error_for_status()?
            .bytes_stream()
            .map_err(MiasmaError::from))
    }
}

#[cfg(test)]
mod test {
    use axum::{Router, routing::get};
    use bytes::BytesMut;
    use tokio::net::TcpListener;

    use super::*;

    async fn test_server(response: String) -> Url {
        let app = Router::new().route("/", get(|| async { response }));

        let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();

        let url = Url::parse(&format!("http://{}", listener.local_addr().unwrap())).unwrap();
        tokio::spawn(async move {
            axum::serve(listener, app).await.unwrap();
        });
        url
    }

    #[tokio::test]
    async fn success() {
        let url = test_server("<poison>".to_owned()).await;
        let client = PoisonClient::new(url, false);

        let stream = client.stream_poison().await;
        let bytes: BytesMut = stream.try_collect().await.unwrap();
        let result = String::from_utf8(bytes.to_vec()).unwrap();

        assert_eq!(result, "&lt;poison&gt;");
    }

    #[tokio::test]
    async fn success_no_escape() {
        let url = test_server("<poison>".to_owned()).await;
        let client = PoisonClient::new(url, true);

        let stream = client.stream_poison().await;
        let bytes: BytesMut = stream.try_collect().await.unwrap();
        let result = String::from_utf8(bytes.to_vec()).unwrap();

        assert_eq!(result, "<poison>");
    }

    #[tokio::test]
    async fn default_on_failure() {
        let client = PoisonClient::new(Url::parse("http://invalid.").unwrap(), false);

        let stream = client.stream_poison().await;
        let bytes: BytesMut = stream.try_collect().await.unwrap();
        let result = String::from_utf8(bytes.to_vec()).unwrap();

        assert!(!result.is_empty());
    }
}
