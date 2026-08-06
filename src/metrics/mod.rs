mod error;
mod metrics_counter;
mod migrations;
mod page;

use std::sync::Arc;

use axum::{Router, routing::get};
pub use error::MetricsError;
pub use metrics_counter::{Metrics, UserAgent};

use tokio::sync::Mutex;

use crate::config::MetricsConfig;

const RESULTS_PER_PAGE: u32 = 50;

#[derive(Clone)]
pub struct MetricsState {
    counter: Arc<Mutex<Metrics>>,
    auth_value: Arc<String>,
}

pub fn new_metrics_router(
    config: Option<&MetricsConfig>,
    metrics: Option<Arc<Mutex<Metrics>>>,
) -> Router {
    let Some(config) = config else {
        return Router::new();
    };
    let encoded_creds = config.encoded_credentials();
    let auth_value = format!("Basic {encoded_creds}");

    Router::new()
        .route(&config.endpoint, get(page::metrics_handler))
        .with_state(MetricsState {
            counter: metrics.expect("metrics should be Some when config is Some"),
            auth_value: Arc::new(auth_value),
        })
}

#[cfg(test)]
mod test {
    use axum::{
        body::Body,
        http::{Request, header, status::StatusCode},
    };
    use base64::prelude::*;
    use tempfile::NamedTempFile;
    use tower::ServiceExt;

    use super::*;

    // Return the temp DB file as well so that it's dropped after the test.
    fn build_test_router() -> (Router, NamedTempFile) {
        let db_file = NamedTempFile::new().unwrap();
        let db_path = db_file.path().to_str().unwrap();
        let config = MetricsConfig::new(db_path, "admin", "admin", "/metrics");
        let metrics = Arc::new(Mutex::new(Metrics::new(db_path.to_owned()).unwrap()));

        let router = new_metrics_router(Some(config).as_ref(), Some(metrics));
        (router, db_file)
    }

    #[tokio::test]
    async fn happy_path() {
        let (router, _db_file) = build_test_router();

        let req = Request::builder()
            .uri("/metrics")
            .header(
                header::AUTHORIZATION,
                format!("Basic {}", BASE64_STANDARD.encode("admin:admin")),
            )
            .body(Body::empty())
            .unwrap();

        let resp = router.oneshot(req).await.unwrap();

        assert_eq!(resp.status(), StatusCode::OK);
    }

    #[tokio::test]
    async fn rejects_unauthenticated_requests() {
        let (router, _db_file) = build_test_router();

        let req = Request::builder()
            .uri("/metrics")
            .body(Body::empty())
            .unwrap();

        let resp = router.oneshot(req).await.unwrap();

        assert_eq!(resp.status(), StatusCode::UNAUTHORIZED);
        assert_eq!(
            resp.headers().get(header::WWW_AUTHENTICATE).unwrap(),
            "Basic"
        );
    }

    #[tokio::test]
    async fn rejects_unauthorized_requests() {
        let (router, _db_file) = build_test_router();

        let req = Request::builder()
            .uri("/metrics")
            .header(
                header::AUTHORIZATION,
                format!(
                    "Basic {}",
                    BASE64_STANDARD.encode("invalid-username:invalid-password")
                ),
            )
            .body(Body::empty())
            .unwrap();

        let resp = router.oneshot(req).await.unwrap();

        assert_eq!(resp.status(), StatusCode::UNAUTHORIZED);
        assert_eq!(
            resp.headers().get(header::WWW_AUTHENTICATE).unwrap(),
            "Basic"
        );
    }
}
