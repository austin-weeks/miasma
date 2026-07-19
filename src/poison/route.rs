use std::sync::Arc;

use axum::{
    body::Body,
    http::{Response, StatusCode},
    response::IntoResponse,
};
use reqwest::header;
use tokio::sync::OwnedSemaphorePermit;

use super::{LinkSettings, gzip, response_stream};
use crate::poison::PoisonClient;

/// Miasma's poison serving trap.
pub async fn serve_poison(
    poison_client: Arc<PoisonClient>,
    in_flight_permit: OwnedSemaphorePermit,
    gzip_response: bool,
    link_settings: LinkSettings,
) -> impl IntoResponse {
    let poison = poison_client.stream_poison().await;

    let stream = response_stream::build_response_stream(poison, link_settings, in_flight_permit);

    let body_stream = if gzip_response {
        Body::from_stream(gzip::gzip_stream(stream))
    } else {
        Body::from_stream(stream)
    };

    let mut builder = Response::builder().header(header::CONTENT_TYPE, "text/html");
    if gzip_response {
        builder = builder.header(header::CONTENT_ENCODING, "gzip");
    }
    builder.body(body_stream).unwrap_or_else(|e| {
        eprintln!("Failed to build poison route response: {e}");
        StatusCode::INTERNAL_SERVER_ERROR.into_response()
    })
}
