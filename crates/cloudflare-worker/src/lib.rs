use std::cell::OnceCell;

use axum::Router;
use miasma::MiasmaConfig;
use tower::ServiceExt;
use worker::{Context, EncodeBody, Env, Error, HttpRequest, Result, event};

thread_local! {
    static ROUTER: OnceCell<Router> = const { OnceCell::new() };
}

#[event(fetch)]
async fn fetch(req: HttpRequest, env: Env, _ctx: Context) -> Result<axum::response::Response> {
    let router = ROUTER.with(|cached| -> Result<Router> {
        if let Some(router) = cached.get() {
            return Ok(router.clone());
        }
        let mut config = MiasmaConfig::builder();
        config.max_in_flight(8);
        if let Ok(source) = env.var("POISON_SOURCE") {
            config.poison_source(
                source
                    .to_string()
                    .parse()
                    .map_err(|e| Error::RustError(format!("{e}")))?,
            );
        }
        let router = miasma::new_miasma_router(config.build())
            .map_err(|e| Error::RustError(e.to_string()))?;
        let _ = cached.set(router.clone());
        Ok(router)
    })?;
    let mut response = match router.oneshot(req).await {
        Ok(response) => response,
        Err(never) => match never {},
    };
    if response.headers().contains_key("content-encoding") {
        response.extensions_mut().insert(EncodeBody::Manual);
    }
    Ok(response)
}
