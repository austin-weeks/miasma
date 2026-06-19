use std::{
    fmt::{self, Display},
    num::ParseIntError,
    str::FromStr,
};

use clap::{ArgMatches, Args, CommandFactory, FromArgMatches, Parser, parser::ValueSource};
use colored::Colorize;
use serde::Deserialize;
use url::Url;

use miasma::MiasmaConfig;

use crate::cli::{
    self,
    config_file::{ConfigFile, load_config_file},
};

const DEFAULT_HOST: &str = "localhost";
const DEFAULT_PORT: u16 = 9999;
const DEFAULT_METRICS_ENDPOINT: &str = "/metrics";

#[derive(Parser, Debug, Clone)]
#[command(
    version,
    about = "Serve an endless maze of poisoned training data. Fight back against AI web scrapers."
)]
pub struct AppArgs {
    /// Load configuration options from a file at the specified path
    #[arg(short = 'f', long)]
    pub config_file: Option<String>,

    /// Port to listen for requests
    #[arg(short = 'p', long, default_value_t = DEFAULT_PORT)]
    #[arg(help_heading = "Server Options")]
    pub port: u16,

    /// Host to listen for requests
    #[arg(long, env = "MIASMA_HOST", default_value_t = DEFAULT_HOST.to_string() )]
    #[arg(help_heading = "Server Options")]
    pub host: String,

    /// Bind to the specified Unix socket rather than a TCP address
    #[cfg(unix)]
    #[arg(long)]
    #[arg(help_heading = "Server Options")]
    pub unix_socket: Option<String>,

    /// Maximum number of in-flight requests - if exceeded, Miasma responds with a 429 error
    #[arg(short = 'c', long, default_value_t = miasma::DEFAULT_MAX_IN_FLIGHT, value_parser = clap::value_parser!(u32).range(1..))]
    pub max_in_flight: u32,

    /// Prefix for embedded links
    #[arg(long, default_value_t = String::from("/"))]
    pub link_prefix: String,

    /// Number of links to include in each response
    #[arg(long, default_value_t = miasma::DEFAULT_LINK_COUNT)]
    pub link_count: u8,

    /// Stop generating links after the scraper reaches the specified depth
    #[arg(long,  default_value_t = MaxDepth::default())]
    pub max_depth: MaxDepth,

    /// Always gzip responses regardless of client's Accept-Encoding header
    #[arg(long, default_value_t = false)]
    pub force_gzip: bool,

    /// Don't escape HTML characters in the poison source's responses
    #[arg(long, default_value_t = false)]
    pub unsafe_allow_html: bool,

    /// Poisoned training data source
    #[arg(long, default_value_t = Url::parse(miasma::DEFAULT_POISON_SOURCE).unwrap())]
    pub poison_source: Url,

    #[command(flatten)]
    pub metrics: Option<MetricsConfig>,
}

/// Handles merging CLI args when options are loaded from a config file.
/// If user provided a CLI flag, it overrides config file values.
struct OverrideMerger(ArgMatches);
impl OverrideMerger {
    fn merge<T>(&self, field: &str, cli_value: T, config_value: Option<T>) -> T {
        // Use config file value if present and not overridden
        if let Some(val) = config_value
            && !self.arg_provided(field)
        {
            val
        } else {
            cli_value
        }
    }

    fn arg_provided(&self, field: &str) -> bool {
        matches!(
            self.0.value_source(field),
            Some(ValueSource::CommandLine | ValueSource::EnvVariable)
        )
    }
}

impl AppArgs {
    pub fn parse_args() -> Self {
        let matches = AppArgs::command().get_matches();
        let args = AppArgs::from_arg_matches(&matches).unwrap_or_else(|e| e.exit());
        let Some(file) = args.config_file.clone() else {
            return args;
        };

        let config = match load_config_file(&file) {
            Ok(c) => c,
            Err(e) => {
                // Mimic clap's error message
                eprintln!("{}: failed to load config file: {e}\n", "error".red());
                eprintln!(
                    "View example configurations at {}",
                    "https://github.com/austin-weeks/miasma/tree/main/docs/config_file/".blue()
                );
                std::process::exit(2);
            }
        };
        let merger = OverrideMerger(matches);
        args.merge_config_file(config, &merger, file)
    }

    fn merge_config_file(
        self,
        config: ConfigFile,
        merger: &OverrideMerger,
        config_file_path: String,
    ) -> Self {
        let metrics_args_provided = merger.arg_provided("metrics_db_path")
            || merger.arg_provided("metrics_username")
            || merger.arg_provided("metrics_password")
            || merger.arg_provided("metrics_endpoint");
        let metrics = match (metrics_args_provided, config.metrics) {
            (true, _) | (false, None) => self.metrics,
            (false, Some(m)) => Some(MetricsConfig {
                metrics_db_path: Some(m.db_path),
                metrics_username: Some(m.username),
                metrics_password: Some(m.password),
                metrics_endpoint: m
                    .endpoint
                    .unwrap_or_else(|| DEFAULT_METRICS_ENDPOINT.to_owned()),
            }),
        };

        Self {
            config_file: Some(config_file_path),
            port: merger.merge("port", self.port, config.server.port),
            host: merger.merge("host", self.host, config.server.host),
            #[cfg(unix)]
            unix_socket: merger.merge(
                "unix_socket",
                self.unix_socket,
                Some(config.server.unix_socket),
            ),
            max_in_flight: merger.merge("max_in_flight", self.max_in_flight, config.max_in_flight),
            link_prefix: merger.merge("link_prefix", self.link_prefix, config.link_prefix),
            link_count: merger.merge("link_count", self.link_count, config.link_count),
            max_depth: merger.merge("max_depth", self.max_depth, config.max_depth),
            force_gzip: merger.merge("force_gzip", self.force_gzip, config.force_gzip),
            unsafe_allow_html: merger.merge(
                "unsafe_allow_html",
                self.unsafe_allow_html,
                config.unsafe_allow_html,
            ),
            poison_source: merger.merge("poison_source", self.poison_source, config.poison_source),
            metrics,
        }
    }

    /// Print configuration information to stderr.
    pub fn print_config_info(&self) {
        let gzip_msg = if self.force_gzip {
            format!(" and {}", "forced gzip compression".blue())
        } else {
            String::new()
        };
        #[cfg(unix)]
        let binding = match &self.unix_socket {
            Some(socket) => socket.blue(),
            None => self.address().blue(),
        };
        #[cfg(not(unix))]
        let binding = self.address().blue();

        eprintln!(
            "Listening on {} with {} max in-flight requests{gzip_msg}...",
            binding,
            self.max_in_flight.to_string().blue()
        );
        eprintln!(
            "Serving poisoned training data from {} at {} with {} links per response and a max depth of {}...",
            self.poison_source.to_string().blue(),
            self.link_prefix.blue(),
            self.link_count.to_string().blue(),
            self.max_depth.to_string().blue(),
        );

        let est_pages_per_bot = match self.max_depth.0 {
            None => "infinite".blue(),
            Some(depth) => cli::page_count_per_bot(self.link_count, depth)
                .map_or_else(|| "too big!".red(), |n| n.to_string().green()),
        };
        eprintln!(
            "Assuming all links are explored, each scraper will consume {est_pages_per_bot} poison pages."
        );

        if self.unsafe_allow_html {
            eprintln!("{} HTML escaping is disabled...", "Warning:".red());
        }

        if let Some(MetricsConfig {
            metrics_db_path: Some(db_path),
            metrics_username: Some(username),
            metrics_endpoint,
            ..
        }) = &self.metrics
        {
            eprintln!(
                "Request metrics are available at {} with credentials {}.",
                metrics_endpoint.blue(),
                format!("{username}:******").blue(),
            );
            eprintln!(
                "Metrics will be written to the SQLite database at {}.",
                db_path.blue(),
            );
        } else {
            eprintln!("Metrics are disabled and will not be collected...");
        }
    }

    /// Get the full 'host:port' address.
    pub fn address(&self) -> String {
        format!("{}:{}", self.host, self.port)
    }

    pub fn to_miasma_config(&self) -> MiasmaConfig {
        let mut builder = MiasmaConfig::builder();
        builder
            .max_in_flight(self.max_in_flight)
            .link_count(self.link_count)
            .link_prefix(&self.link_prefix)
            .force_gzip(self.force_gzip)
            .unsafe_allow_html(self.unsafe_allow_html)
            .poison_source(self.poison_source.clone());

        if let Some(d) = self.max_depth.0 {
            builder.max_depth(d);
        }
        if let Some(MetricsConfig {
            metrics_db_path: Some(db_path),
            metrics_username: Some(username),
            metrics_password: Some(password),
            metrics_endpoint,
        }) = &self.metrics
        {
            builder.metrics(db_path, metrics_endpoint, username, password);
        }

        builder.build()
    }
}

#[derive(Debug, Clone, Deserialize, PartialEq, Default)]
pub struct MaxDepth(pub Option<u32>);

impl Display for MaxDepth {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self.0 {
            Some(v) => f.write_str(&v.to_string()),
            None => f.write_str("none"),
        }
    }
}

impl FromStr for MaxDepth {
    type Err = ParseIntError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if s.eq_ignore_ascii_case("none") {
            return Ok(Self(None));
        }
        match s.parse::<u32>() {
            Ok(v) => Ok(Self(Some(v))),
            Err(e) => Err(e),
        }
    }
}

/// If this value is Some, all sub-fields will also be Some
#[derive(Args, Debug, Clone)]
#[expect(clippy::struct_field_names)]
pub struct MetricsConfig {
    #[expect(clippy::doc_markdown)]
    /// Path to SQLite database file (e.g. 'miasma.db')
    #[arg(long, requires_all = ["metrics_username", "metrics_password"])]
    #[arg(help_heading = "Metrics Options")]
    pub metrics_db_path: Option<String>,

    /// Basic auth username required to access request metrics
    #[arg(long, requires = "metrics_db_path")]
    #[arg(help_heading = "Metrics Options")]
    pub metrics_username: Option<String>,

    /// Basic auth password required to access request metrics
    #[arg(long, requires = "metrics_db_path")]
    #[arg(help_heading = "Metrics Options")]
    pub metrics_password: Option<String>,

    /// Endpoint at which request metrics will be served
    #[arg(
        long, default_value = DEFAULT_METRICS_ENDPOINT,
        requires_all = ["metrics_db_path", "metrics_username", "metrics_password"],
    )]
    #[arg(help_heading = "Metrics Options")]
    pub metrics_endpoint: String,
}

#[cfg(test)]
mod test {
    use super::*;
    use base64::prelude::*;

    #[test]
    fn to_miasma_config() {
        let poison_source = Url::parse("https://example.com").unwrap();
        let args = AppArgs {
            #[cfg(unix)]
            unix_socket: None,
            port: 0,
            host: String::new(),

            max_in_flight: 8,
            link_prefix: "/bots".to_owned(),
            link_count: 8,
            max_depth: MaxDepth(Some(8)),
            poison_source: poison_source.clone(),
            force_gzip: true,
            unsafe_allow_html: true,

            metrics: Some(MetricsConfig {
                metrics_db_path: Some("miasma.db".to_owned()),
                metrics_username: Some("admin".to_owned()),
                metrics_password: Some("admin".to_owned()),
                metrics_endpoint: "/serve-metrics".to_owned(),
            }),

            config_file: None,
        };

        let config = args.to_miasma_config();

        assert_eq!(config.max_in_flight, 8);
        assert_eq!(config.link_prefix.to_string(), "/bots/".to_owned());
        assert_eq!(config.link_count, 8);
        assert_eq!(config.max_depth, Some(8));
        assert_eq!(config.poison_source, poison_source);
        assert!(config.force_gzip);
        assert!(config.unsafe_allow_html);

        let metrics = config.metrics.unwrap();
        assert_eq!(metrics.db_path, "miasma.db".to_owned());
        assert_eq!(metrics.endpoint, "/serve-metrics".to_owned());
        assert_eq!(
            metrics.encoded_credentials(),
            BASE64_STANDARD.encode("admin:admin")
        );
    }

    #[test]
    fn args_address() {
        let config = AppArgs {
            #[cfg(unix)]
            unix_socket: None,
            port: 8080,
            host: "127.0.0.1".to_owned(),

            config_file: None,
            max_in_flight: Default::default(),
            link_prefix: "/".into(),
            link_count: miasma::DEFAULT_LINK_COUNT,
            max_depth: MaxDepth::default(),
            force_gzip: bool::default(),
            unsafe_allow_html: bool::default(),
            poison_source: Url::parse("https://example.com").unwrap(),
            metrics: None,
        };

        assert_eq!(config.address(), "127.0.0.1:8080");
    }

    // Because env variables are modified, tests need to run in parallel
    #[serial_test::serial]
    mod config_file_merging {
        use super::*;
        use crate::cli::config_file::{MetricsFileConfig, ServerFileConfig};
        use std::assert_matches;

        #[test]
        fn no_args_with_empty_config_file_uses_default_values() {
            let matches = AppArgs::command()
                .try_get_matches_from(Vec::<&str>::default())
                .unwrap();
            let args = AppArgs::from_arg_matches(&matches).unwrap();
            let config = ConfigFile::default();

            let config_file_path = "config.yaml".to_owned();
            let result =
                args.merge_config_file(config, &OverrideMerger(matches), config_file_path.clone());
            assert_eq!(result.config_file, Some(config_file_path));
            assert_eq!(result.port, DEFAULT_PORT);
            assert_eq!(result.host, DEFAULT_HOST.to_owned());
            #[cfg(unix)]
            assert_eq!(result.unix_socket, None);
            assert_eq!(result.max_in_flight, miasma::DEFAULT_MAX_IN_FLIGHT);
            assert_eq!(result.link_prefix, "/".to_owned());
            assert_eq!(result.link_count, miasma::DEFAULT_LINK_COUNT);
            assert_eq!(result.max_depth, MaxDepth::default());
            assert!(!result.force_gzip);
            assert!(!result.unsafe_allow_html);
            assert_matches!(result.metrics, None);
            assert_eq!(
                result.poison_source,
                Url::parse(miasma::DEFAULT_POISON_SOURCE).unwrap()
            );
        }

        #[test]
        fn no_extra_args_uses_config_file_values() {
            let matches = AppArgs::command()
                .try_get_matches_from(Vec::<&str>::default())
                .unwrap();
            let args = AppArgs::from_arg_matches(&matches).unwrap();
            let config = ConfigFile {
                link_prefix: Some("/bots".into()),
                link_count: Some(12),
                max_in_flight: Some(99),
                max_depth: Some(MaxDepth(Some(4))),
                force_gzip: Some(true),
                unsafe_allow_html: Some(true),
                poison_source: Some(Url::parse("https://github.com").unwrap()),
                server: ServerFileConfig {
                    host: Some("test-host".into()),
                    port: Some(7000),
                    unix_socket: Some("test.sock".into()),
                },
                metrics: Some(MetricsFileConfig {
                    db_path: "test.db".into(),
                    username: "test".into(),
                    password: "test".into(),
                    endpoint: Some("/test".into()),
                }),
            };

            let result = args.merge_config_file(config, &OverrideMerger(matches), String::new());
            assert_eq!(result.link_prefix, "/bots");
            assert_eq!(result.link_count, 12);
            assert_eq!(result.max_in_flight, 99);
            assert_matches!(result.max_depth, MaxDepth(Some(4)));
            assert!(result.force_gzip);
            assert!(result.unsafe_allow_html);
            assert_eq!(result.poison_source.as_str(), "https://github.com/");
            assert_eq!(result.host, "test-host");
            assert_eq!(result.port, 7000);
            #[cfg(unix)]
            assert_eq!(result.unix_socket, Some("test.sock".into()));
            let metrics = result.metrics.unwrap();
            assert_eq!(metrics.metrics_db_path, Some("test.db".into()));
            assert_eq!(metrics.metrics_username, Some("test".into()));
            assert_eq!(metrics.metrics_password, Some("test".into()));
            assert_eq!(metrics.metrics_endpoint, "/test");
        }

        #[test]
        fn extra_args_override_config_file_values() {
            let flags = [
                "miasma",
                "--link-prefix",
                "/bots",
                "--link-count",
                "12",
                "--max-in-flight",
                "99",
                "--max-depth",
                "4",
                "--force-gzip",
                "--unsafe-allow-html",
                "--poison-source",
                "https://example.com",
                "--host",
                "test-host",
                "--port",
                "7000",
                #[cfg(unix)]
                "--unix-socket",
                #[cfg(unix)]
                "test.sock",
            ];

            let matches = AppArgs::command().try_get_matches_from(flags).unwrap();
            let args = AppArgs::from_arg_matches(&matches).unwrap();
            let config = ConfigFile {
                link_prefix: Some("BAD".into()),
                link_count: Some(0),
                max_in_flight: Some(0),
                max_depth: Some(MaxDepth(Some(0))),
                force_gzip: Some(false),
                unsafe_allow_html: Some(false),
                poison_source: Some(Url::parse("https://BAD.com").unwrap()),
                server: ServerFileConfig {
                    host: Some("BAD".into()),
                    port: Some(0),
                    unix_socket: Some("BAD".into()),
                },
                ..Default::default()
            };
            let result = args.merge_config_file(config, &OverrideMerger(matches), String::new());
            assert_eq!(result.link_prefix, "/bots");
            assert_eq!(result.link_count, 12);
            assert_eq!(result.max_in_flight, 99);
            assert_matches!(result.max_depth, MaxDepth(Some(4)));
            assert!(result.force_gzip);
            assert!(result.unsafe_allow_html);
            assert_eq!(result.poison_source.as_str(), "https://example.com/");
            assert_eq!(result.host, "test-host");
            assert_eq!(result.port, 7000);
            #[cfg(unix)]
            assert_eq!(result.unix_socket, Some("test.sock".to_owned()));
        }

        #[test]
        fn env_vars_override_config_file_values() {
            temp_env::with_var("MIASMA_HOST", Some("test-host"), || {
                let matches = AppArgs::command()
                    .try_get_matches_from(Vec::<&str>::default())
                    .unwrap();
                let args = AppArgs::from_arg_matches(&matches).unwrap();
                let config = ConfigFile {
                    server: ServerFileConfig {
                        host: Some("BAD".into()),
                        ..Default::default()
                    },
                    ..Default::default()
                };

                let result =
                    args.merge_config_file(config, &OverrideMerger(matches), String::new());
                assert_eq!(result.host, "test-host");
            });
        }

        #[test]
        fn any_metrics_args_override_config_file() {
            let flags = [
                "miasma",
                "--metrics-db-path",
                "miasma.db",
                "--metrics-username",
                "admin",
                "--metrics-password",
                "password",
                "--metrics-endpoint",
                "/metrics",
            ];

            let matches = AppArgs::command().try_get_matches_from(flags).unwrap();
            let args = AppArgs::from_arg_matches(&matches).unwrap();
            let config = ConfigFile {
                metrics: Some(MetricsFileConfig {
                    db_path: "BAD".into(),
                    username: "BAD".into(),
                    password: "BAD".into(),
                    endpoint: Some("BAD".into()),
                }),
                ..Default::default()
            };
            let result = args.merge_config_file(config, &OverrideMerger(matches), String::new());

            let metrics = result.metrics.unwrap();
            assert_eq!(metrics.metrics_db_path, Some("miasma.db".into()));
            assert_eq!(metrics.metrics_username, Some("admin".into()));
            assert_eq!(metrics.metrics_password, Some("password".into()));
            assert_eq!(metrics.metrics_endpoint, "/metrics");
        }
    }
}
