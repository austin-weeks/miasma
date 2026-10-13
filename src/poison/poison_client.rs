use std::time::Duration;

#[cfg(target_arch = "wasm32")]
use super::wasm_breaker::WasmBreaker;
#[cfg(not(target_arch = "wasm32"))]
use failsafe::{
    StateMachine,
    backoff::EqualJittered,
    failure_policy::{ConsecutiveFailures, OrElse, SuccessRateOverTimeWindow},
    futures::CircuitBreaker,
};
use futures::{StreamExt, TryStreamExt};
use reqwest::Client;
#[cfg(target_arch = "wasm32")]
use std::sync::Mutex;
use url::Url;

use crate::{
    MIASMA_USER_AGENT, MiasmaError, MiasmaStream, poison::HtmlEscapeMode,
    utils::html_escaper::escape_html_stream,
};

pub struct PoisonClient {
    client: Client,
    #[cfg(not(target_arch = "wasm32"))]
    breaker: StateMachine<
        OrElse<SuccessRateOverTimeWindow<EqualJittered>, ConsecutiveFailures<EqualJittered>>,
        (),
    >,
    #[cfg(target_arch = "wasm32")]
    breaker: Mutex<WasmBreaker>,
    poison_source: Url,
    escape_mode: HtmlEscapeMode,
}

impl PoisonClient {
    const REQUEST_TIMEOUT: Duration = Duration::from_secs(20);

    pub fn new(poison_source: Url, escape_mode: HtmlEscapeMode) -> Self {
        let client = Client::builder().user_agent(MIASMA_USER_AGENT);
        #[cfg(not(target_arch = "wasm32"))]
        let client = client.gzip(true).timeout(Self::REQUEST_TIMEOUT);
        let client = client.build().expect("should be able to build client");

        #[cfg(not(target_arch = "wasm32"))]
        let breaker = failsafe::Config::new().build();
        #[cfg(target_arch = "wasm32")]
        let breaker = Mutex::new(WasmBreaker::default());

        Self {
            client,
            breaker,
            poison_source,
            escape_mode,
        }
    }

    /// Fetch poisoned training data.
    pub async fn stream_poison(&self) -> Option<impl MiasmaStream + use<>> {
        #[cfg(not(target_arch = "wasm32"))]
        let result = self
            .breaker
            .call(
                // Create the request only if the breaker permits it.
                async { self.fetch_poison().await },
            )
            .await;

        #[cfg(not(target_arch = "wasm32"))]
        if let Err(failsafe::Error::Inner(e)) = &result {
            // The error message is already well formatted
            eprintln!("{e}");
        }

        #[cfg(target_arch = "wasm32")]
        let result = {
            if !self
                .breaker
                .lock()
                .expect("breaker mutex poisoned")
                .permits(js_sys::Date::now())
            {
                return None;
            }
            let result = self.fetch_poison().await;
            self.breaker
                .lock()
                .expect("breaker mutex poisoned")
                .record(result.is_err(), js_sys::Date::now());
            result
        };

        let Ok(poison_stream) = result else {
            return None;
        };

        Some(match self.escape_mode {
            HtmlEscapeMode::Escape => escape_html_stream(poison_stream).left_stream(),
            HtmlEscapeMode::NoEscape => poison_stream.right_stream(),
        })
    }

    async fn fetch_poison(&self) -> Result<impl MiasmaStream + use<>, MiasmaError> {
        let request = self.client.get(self.poison_source.as_str());
        #[cfg(target_arch = "wasm32")]
        let request = request.timeout(Self::REQUEST_TIMEOUT);
        let fetch = request.send();
        // Fetch futures and streams remain on the same WASM thread.
        #[cfg(target_arch = "wasm32")]
        let fetch = send_wrapper::SendWrapper::new(fetch);
        let response = fetch.await?.error_for_status()?.bytes_stream();
        #[cfg(target_arch = "wasm32")]
        let response = send_wrapper::SendWrapper::new(response);
        Ok(response.map_err(MiasmaError::from))
    }
}

#[cfg(test)]
mod test {
    use axum::{Router, routing::get};
    use bytes::BytesMut;

    use crate::test_utils::{self, TestServer};

    use super::*;

    async fn test_server_with_response(response: String) -> TestServer {
        test_utils::test_server(Router::new().route("/", get(|| async { response }))).await
    }

    #[tokio::test]
    async fn success() {
        let server = test_server_with_response("<poison>".to_owned()).await;
        let client = PoisonClient::new(server.url, HtmlEscapeMode::Escape);

        let stream = client.stream_poison().await.unwrap();
        let bytes: BytesMut = stream.try_collect().await.unwrap();
        let result = String::from_utf8(bytes.to_vec()).unwrap();

        assert_eq!(result, "&lt;poison&gt;");
    }

    #[tokio::test]
    async fn success_no_escape() {
        let server = test_server_with_response("<poison>".to_owned()).await;
        let client = PoisonClient::new(server.url, HtmlEscapeMode::NoEscape);

        let stream = client.stream_poison().await.unwrap();
        let bytes: BytesMut = stream.try_collect().await.unwrap();
        let result = String::from_utf8(bytes.to_vec()).unwrap();

        assert_eq!(result, "<poison>");
    }

    #[tokio::test]
    async fn none_on_failure() {
        let client = PoisonClient::new(
            Url::parse("http://invalid.").unwrap(),
            HtmlEscapeMode::Escape,
        );

        let stream = client.stream_poison().await;
        assert!(stream.is_none());
    }
}
