use std::io;

use thiserror::Error;

use crate::metrics::MetricsError;

/// Errors that may occur when initializing or running Miasma.
#[derive(Error, Debug)]
pub enum MiasmaError {
    #[error("failed to fetch poison: {0}")]
    PoisonFetch(#[from] reqwest::Error),
    #[cfg(unix)]
    #[error("failed to create unix socket listener: {0}")]
    UnixSocketBind(io::Error),
    #[error("failed to create TCP listener: {0}")]
    TcpBind(io::Error),
    #[error("miasma's server exited unexpectedly: {0}")]
    ServerRuntime(io::Error),
    #[error(transparent)]
    Metrics(#[from] MetricsError),
}
