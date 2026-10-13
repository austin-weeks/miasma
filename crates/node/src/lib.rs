use std::fmt::Display;

use axum::Router;
use miasma::MiasmaConfig;
use tower::ServiceExt;
use wasm_bindgen::prelude::*;
use worker::js_sys;

#[wasm_bindgen]
pub struct Miasma {
    router: Router,
}

#[wasm_bindgen]
impl Miasma {
    #[wasm_bindgen(constructor)]
    pub fn new(poison_source: &str, max_in_flight: u32) -> Result<Self, JsValue> {
        let mut config = MiasmaConfig::builder();
        config.max_in_flight(max_in_flight);
        if !poison_source.is_empty() {
            config.poison_source(poison_source.parse().map_err(js_error)?);
        }
        Ok(Self {
            router: miasma::new_miasma_router(config.build()).map_err(js_error)?,
        })
    }

    pub async fn fetch(
        &self,
        request: worker::web_sys::Request,
    ) -> Result<worker::web_sys::Response, JsValue> {
        let request = worker::request_from_wasm(request).map_err(js_error)?;
        let response = match self.router.clone().oneshot(request).await {
            Ok(response) => response,
            Err(never) => match never {},
        };
        worker::response_to_wasm(response).map_err(js_error)
    }
}

fn js_error(error: impl Display) -> JsValue {
    js_sys::Error::new(&error.to_string()).into()
}
