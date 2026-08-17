use std::fmt::Display;

use base64::prelude::*;
use derive_builder::Builder;
use url::Url;

use crate::{DEFAULT_LINK_COUNT, DEFAULT_MAX_IN_FLIGHT, DEFAULT_POISON_SOURCE};

/// Defines configuration settings for miasma.
///
/// ## Usage
///
/// Use default configuration:
///
/// ```
/// use miasma::MiasmaConfig;
///
/// let config = MiasmaConfig::default();
/// ```
///
/// Customize configuration:
///
/// ```
/// use miasma::MiasmaConfig;
///
/// let config = MiasmaConfig::builder()
///     .max_in_flight(20)
///     .link_prefix("/bots")
///     .link_count(8)
///     .max_depth(10)
///     .force_gzip(true)
///     .metrics(
///         "/data/miasma.db",
///         "/request-metrics",
///         "admin",
///         "admin",
///     )
///     .build();
/// ```
#[derive(Builder)]
#[builder(build_fn(name = "unsafe_build", private))]
pub struct MiasmaConfig {
    /// Maximum number of in-flight requests. If exceeded, Miasma responds with a 429 error.
    #[builder(default = DEFAULT_MAX_IN_FLIGHT)]
    pub max_in_flight: u32,

    /// Prefix for embedded links.
    #[builder(default = LinkPrefix::new("/"))]
    #[builder(setter(custom))]
    pub link_prefix: LinkPrefix,

    /// Number of links to include in each response.
    #[builder(default = DEFAULT_LINK_COUNT)]
    pub link_count: u8,

    /// Stop generating links after the scraper reaches the specified depth.
    #[builder(default = None)]
    #[builder(setter(strip_option))]
    pub max_depth: Option<u32>,

    /// Always gzip responses regardless of client's Accept-Encoding header.
    #[builder(default = false)]
    pub force_gzip: bool,

    /// Don't escape HTML characters in the poison source's responses.
    #[builder(default = false)]
    pub unsafe_allow_html: bool,

    /// Poisoned training data source.
    #[builder(default = Url::parse(DEFAULT_POISON_SOURCE).unwrap())]
    pub poison_source: Url,

    /// Settings for Miasma's requests metrics feature.
    #[builder(default = None)]
    #[builder(setter(custom))]
    pub metrics: Option<MetricsConfig>,

    /// Disable poison source response caching.
    #[builder(default = false)]
    pub no_poison_cache: bool,
}

impl MiasmaConfig {
    /// Returns a new fluent builder.
    pub fn builder() -> MiasmaConfigBuilder {
        MiasmaConfigBuilder::default()
    }
}

impl Default for MiasmaConfig {
    fn default() -> Self {
        Self::builder().build()
    }
}

impl MiasmaConfigBuilder {
    /// Enables the optional requests metrics feature.
    ///
    /// - `db_path` - path to `SQLite` database file where request metrics will be stored.
    /// - `endpoint` - endpoint at which the router should server request metrics.
    /// - `username` - basic auth username required to access request metrics.
    /// - `password` - basic auth password required to access request metrics.
    pub fn metrics(
        &mut self,
        db_path: &str,
        endpoint: &str,
        username: &str,
        password: &str,
    ) -> &mut Self {
        self.metrics = Some(Some(MetricsConfig::new(
            db_path, username, password, endpoint,
        )));
        self
    }

    /// Prefix for embedded links.
    ///
    /// This should be the absolute path to where your application serves Miasma's routes.
    /// For example, if you nest Miasma's router at `/bots`, but your application is served
    /// at `/api/my-app/`, `link_prefix` should be set to `/api/my-app/bots`.
    pub fn link_prefix(&mut self, link_prefix: &str) -> &mut Self {
        self.link_prefix = Some(LinkPrefix::new(link_prefix));
        self
    }

    /// Build the configured `MiasmaConfig`.
    #[expect(clippy::missing_panics_doc)]
    pub fn build(&self) -> MiasmaConfig {
        self.unsafe_build()
            .expect("build only errors for unset required fields - all config fields are optional")
    }
}

#[derive(Debug, Clone)]
#[cfg_attr(test, derive(PartialEq))]
pub struct MetricsConfig {
    pub db_path: String,
    pub endpoint: String,
    username: String,
    password: String,
}

impl MetricsConfig {
    pub fn new(db_path: &str, username: &str, password: &str, endpoint: &str) -> Self {
        MetricsConfig {
            db_path: db_path.to_owned(),
            endpoint: endpoint.to_owned(),
            username: username.to_owned(),
            password: password.to_owned(),
        }
    }
    /// Returns credentials as a base 64 encoded string.
    pub fn encoded_credentials(&self) -> String {
        BASE64_STANDARD.encode(format!("{}:{}", self.username, self.password))
    }
}

/// Link prefix validated to start and end with '/'
#[derive(Clone)]
#[cfg_attr(test, derive(Debug, PartialEq))]
pub struct LinkPrefix(String);

impl LinkPrefix {
    fn new(prefix: &str) -> Self {
        let mut prefix = prefix.to_owned();
        if !prefix.starts_with('/') {
            prefix.insert(0, '/');
        }
        if !prefix.ends_with('/') {
            prefix.push('/');
        }
        Self(prefix)
    }
}

impl Display for LinkPrefix {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn link_prefix_validation() {
        let cases = [
            ("/", "/"),
            ("foo", "/foo/"),
            ("/foo", "/foo/"),
            ("foo/", "/foo/"),
            ("/foo/", "/foo/"),
        ];

        for (input, expected) in cases {
            let prefix = LinkPrefix::new(input);
            assert_eq!(prefix.0, expected);
        }
    }

    #[test]
    fn builder_build_with_default_values() {
        // Ensure the .expect() call does not trigger
        MiasmaConfig::builder().build();
    }
}
