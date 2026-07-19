use std::sync::Arc;

use axum::{
    Router,
    body::Body,
    extract::{Query, Request, State, rejection::QueryRejection},
    response::{IntoResponse, Response},
    routing::get,
};
use reqwest::{StatusCode, header};
use serde::Deserialize;
use tokio::sync::{Mutex, Semaphore, TryAcquireError};

use crate::{
    MiasmaConfig, MiasmaError,
    metrics::{self, Metrics},
    poison::{self, LinkSettings, PoisonClient},
};

#[derive(Deserialize)]
pub struct QueryParams {
    /// We use 'page' instead of depth to look more convincing to scrapers.
    page: Option<u32>,
}
impl QueryParams {
    pub const CURRENT_DEPTH_QUERY_PARAM: &str = "page";
}

#[derive(Clone)]
pub struct AppState {
    metrics: Option<Arc<Mutex<Metrics>>>,
    poison_client: Arc<PoisonClient>,
    config: Arc<MiasmaConfig>,
    in_flight_sem: Arc<Semaphore>,
}

/// Build a new axum `Router` serving Miasma's routes.
///
/// # Usage
///
/// ```
/// use miasma::MiasmaConfig;
/// use axum::{Router, routing::get};
///
/// let config = MiasmaConfig::builder()
///     .link_prefix("/bots")
///     .build();
///
/// let miasma_router = miasma::new_miasma_router(config).unwrap();
///
/// let my_router = Router::new()
///     .route("/", get(|| async { "ok" }))
///     .nest("/bots", miasma_router);
/// ```
///
/// # Errors
///
/// Returns an error if the metrics feature is enabled and initializing the
/// database fails.
pub fn new_miasma_router(config: MiasmaConfig) -> Result<Router, MiasmaError> {
    let metrics = match &config.metrics {
        None => None,
        Some(c) => {
            let metrics = Metrics::new(c.db_path.clone())?;
            Some(Arc::new(Mutex::new(metrics)))
        }
    };
    let metrics_router = metrics::new_metrics_router(config.metrics.as_ref(), metrics.clone());
    let router = Router::new()
        .fallback(get(app_handler))
        .with_state(AppState {
            metrics,
            in_flight_sem: Arc::new(Semaphore::new(config.max_in_flight as usize)),
            poison_client: Arc::new(PoisonClient::new(
                config.poison_source.clone(),
                config.unsafe_allow_html,
            )),
            config: Arc::new(config),
        })
        .merge(metrics_router);
    Ok(router)
}

#[axum::debug_handler(state = AppState)]
async fn app_handler(
    State(state): State<AppState>,
    query: Result<Query<QueryParams>, QueryRejection>,
    req: Request,
) -> impl IntoResponse {
    let in_flight_permit = match state.in_flight_sem.try_acquire_owned() {
        Ok(p) => p,
        Err(TryAcquireError::NoPermits) => {
            return Response::builder()
                .status(StatusCode::TOO_MANY_REQUESTS)
                .header(header::RETRY_AFTER, 5)
                .body(Body::empty())
                .unwrap();
        }
        Err(TryAcquireError::Closed) => {
            return StatusCode::INTERNAL_SERVER_ERROR.into_response();
        }
    };

    let headers = req.headers();

    let gzip_response = state.config.force_gzip
        || headers.get(header::ACCEPT_ENCODING).is_some_and(|acc| {
            acc.to_str()
                .unwrap_or("")
                .split(',')
                // Don't you dare allocate anything !
                .any(|tok| tok.trim().eq_ignore_ascii_case("gzip"))
        });

    let current_depth = query.ok().and_then(|q| q.page).unwrap_or(1);

    if let Some(counter) = state.metrics {
        let user_agent = headers
            .get(header::USER_AGENT)
            .map_or("NO-USER-AGENT", |ua| {
                ua.to_str().unwrap_or("INVALID-USER-AGENT-STRING")
            });
        counter.lock().await.count_request(user_agent);
    }

    let link_settings = LinkSettings::next(&state.config, current_depth);

    poison::serve_poison(
        state.poison_client,
        in_flight_permit,
        gzip_response,
        link_settings,
    )
    .await
    .into_response()
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::{
        body::Body,
        http::{Request, StatusCode, header::RETRY_AFTER},
    };
    use serial_test::serial;
    use tower::ServiceExt;
    use url::Url;

    #[tokio::test]
    #[serial]
    async fn happy_path() {
        let app = new_miasma_router(MiasmaConfig {
            max_in_flight: 1,
            poison_source: Url::parse("https://example.com").unwrap(),
            ..Default::default()
        })
        .unwrap();

        let response = app
            .oneshot(Request::builder().uri("/").body(Body::empty()).unwrap())
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::OK);
    }

    #[tokio::test]
    #[serial]
    #[allow(clippy::similar_names)]
    async fn returns_429_when_max_in_flight_reached() {
        let app = new_miasma_router(MiasmaConfig {
            max_in_flight: 1,
            poison_source: Url::parse("https://example.com").unwrap(),
            ..Default::default()
        })
        .unwrap();
        let req_1 = Request::builder().uri("/").body(Body::empty()).unwrap();
        let req_2 = Request::builder().uri("/").body(Body::empty()).unwrap();

        let (res_1, res_2) = tokio::join!(app.clone().oneshot(req_1), app.oneshot(req_2));

        let res1 = res_1.unwrap();
        let res2 = res_2.unwrap();

        let limited = if res1.status() == StatusCode::TOO_MANY_REQUESTS {
            res1
        } else if res2.status() == StatusCode::TOO_MANY_REQUESTS {
            res2
        } else {
            panic!(
                "expected one 429, got {} and {}",
                res1.status(),
                res2.status()
            );
        };

        assert_eq!(limited.status(), StatusCode::TOO_MANY_REQUESTS);
        assert_eq!(
            limited
                .headers()
                .get(RETRY_AFTER)
                .and_then(|v| v.to_str().ok()),
            Some("5")
        );
    }
}
