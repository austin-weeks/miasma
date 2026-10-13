use std::io;

use thiserror::Error;

#[cfg(not(target_arch = "wasm32"))]
use crate::metrics::MetricsError;

/// Errors that may occur when initializing or running Miasma.
#[derive(Error, Debug)]
pub enum MiasmaError {
    #[error("max_in_flight exceeds the platform limit of {0}")]
    MaxInFlightExceeded(usize),
    #[error("failed to fetch poison: {0:?}")]
    PoisonFetch(#[from] reqwest::Error),
    #[cfg(unix)]
    #[error("failed to create unix socket listener: {0}")]
    UnixSocketBind(io::Error),
    #[error("failed to create TCP listener: {0}")]
    TcpBind(io::Error),
    #[error("miasma's server exited unexpectedly: {0}")]
    ServerRuntime(io::Error),
    #[error(transparent)]
    #[cfg(not(target_arch = "wasm32"))]
    Metrics(#[from] MetricsError),
}
