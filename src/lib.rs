//! # 🌀 Miasma
//!
//! AI companies continually scrape the internet at an enormous scale, swallowing
//! up all of its contents to use as training data for their next models. If you
//! have a public website, they are already stealing your work.
//!
//! Miasma is here to help you fight back! Serve Miasma's routes and point any malicious
//! traffic towards it. Miasma will send poisoned training data alongside multiple
//! self-referential links. It's an endless buffet of slop for the slop machines.
//!
//! For more information on how to trap scrapers, see the [project README](https://github.com/austin-weeks/miasma).
//!
//! ## Licensing Considerations
//!
//! Miasma is licensed under GPL-v3, a strong copyleft license requiring
//! derivative works be released as free software under the same license.
//!
//! Simply using this library does _not_ require you to release your source code. However,
//! if you distribute an application that incorporates this library, GPLv3 may
//! require the combined work to be licensed under GPL-v3 and that the accompanying
//! source code be released as free software.
//!
//! For more information, see the full [GPL-v3 text](https://www.gnu.org/licenses/gpl-3.0.en.html).
//!
//! ## Usage
//! ```
//! // Set up Miasma's axum router
//! use miasma::MiasmaConfig;
//!
//! let config = MiasmaConfig::builder()
//!     .link_prefix("/bots")
//!     .build();
//! let miasma_router = miasma::new_miasma_router(config).unwrap();
//!
//! // Nest the routes within your application
//! use axum::{Router, routing::get};
//!
//! let my_router = Router::new()
//!     .route("/", get(|| async { "ok" }))
//!     .nest("/bots", miasma_router);
//!
//! // Start the server
//! use tokio::net::TcpListener;
//!
//! # async {
//! let listener = TcpListener::bind("0.0.0.0:3000").await.unwrap();
//! axum::serve(listener, my_router).await.unwrap();
//! # };
//! ```
//!
//! ## Gotchas
//!
//! Be sure to set Miasma's `link_prefix` to be the same endpoint that you nest its
//! router. This ensures that generated links point back to the correct location on
//! your site.
//!
//! If your application is served behind a stripped path, include this in the `link_prefix`.
//! For example, if you nest Miasma's router at `/bots`, but your application is served
//! at `/api/my-app/`, the `link_prefix` should be set to `/api/my-app/bots`.

mod config;
mod error;
mod metrics;
mod poison;
mod response_templates;
mod router;
#[doc(hidden)]
pub mod templating;
mod utils;

#[cfg(test)]
pub mod test_utils;

use router::QueryParams;

pub use config::{MiasmaConfig, MiasmaConfigBuilder};
pub use error::MiasmaError;
pub use router::new_miasma_router;

use bytes::Bytes;
use futures::Stream;

#[doc(hidden)]
pub const MIASMA_USER_AGENT: &str = concat!(
    env!("CARGO_PKG_NAME"),
    "/",
    env!("CARGO_PKG_VERSION"),
    " (github.com/austin-weeks/miasma)"
);

#[doc(hidden)]
pub const DEFAULT_POISON_SOURCE: &str = "https://rnsaffn.com/poison2/?mask=0";
#[doc(hidden)]
pub const DEFAULT_LINK_COUNT: u8 = 5;
#[doc(hidden)]
pub const DEFAULT_MAX_IN_FLIGHT: u32 = 500;

/// Alias for Stream of `Result<Bytes, E>`
#[doc(hidden)]
pub trait MiasmaStream<E = MiasmaError>: Stream<Item = Result<Bytes, E>> {}
impl<T, E> MiasmaStream<E> for T where T: Stream<Item = Result<Bytes, E>> {}
