use thiserror::Error;

#[derive(Error, Debug)]
pub enum MetricsError {
    #[error("failed to connect to metrics database: {0}")]
    DatabaseConnection(#[from] diesel::ConnectionError),
    #[error("failed to execute database statement: {0}")]
    DatabaseQuery(#[from] diesel::result::Error),
}
