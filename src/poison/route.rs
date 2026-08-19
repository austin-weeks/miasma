use axum::{
    body::Body,
    http::{Response, StatusCode},
    response::IntoResponse,
};
use reqwest::header;

use crate::poison::{
    gzip,
    response_stream::{self, PoisonResponseStreamArgs},
};

/// Miasma's poison serving trap.
pub async fn serve_poison(
    poison_stream_args: PoisonResponseStreamArgs,
    gzip_response: bool,
) -> impl IntoResponse {
    let stream = response_stream::build_response_stream(poison_stream_args);

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
