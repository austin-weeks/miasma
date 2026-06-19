use serde::Deserialize;
use thiserror::Error;
use url::Url;

use crate::cli::MaxDepth;

#[derive(Error, Debug)]
pub enum ConfigFileError {
    #[error(transparent)]
    Yaml(serde_yaml::Error),
    #[error(transparent)]
    Json(serde_json::Error),
    #[error(transparent)]
    Toml(toml::de::Error),
    #[error("config language not supported")]
    UnsupportedLanguage,
    #[error(transparent)]
    FileRead(#[from] std::io::Error),
}

#[derive(Deserialize)]
#[cfg_attr(test, derive(Default))]
pub struct ConfigFile {
    pub max_in_flight: Option<u32>,
    pub link_prefix: Option<String>,
    pub link_count: Option<u8>,
    pub max_depth: Option<MaxDepth>,
    pub force_gzip: Option<bool>,
    pub unsafe_allow_html: Option<bool>,
    pub poison_source: Option<Url>,
    #[serde(default)]
    pub server: ServerFileConfig,
    pub metrics: Option<MetricsFileConfig>,
}
#[derive(Deserialize, Default)]
pub struct ServerFileConfig {
    pub host: Option<String>,
    pub port: Option<u16>,
    pub unix_socket: Option<String>,
}
#[derive(Deserialize)]
pub struct MetricsFileConfig {
    pub db_path: String,
    pub username: String,
    pub password: String,
    pub endpoint: Option<String>,
}

pub fn load_config_file(file_path: &str) -> Result<ConfigFile, ConfigFileError> {
    let file_contents = std::fs::read_to_string(file_path)?;
    let file = file_path.to_lowercase();
    #[expect(clippy::case_sensitive_file_extension_comparisons)]
    let config = if file.ends_with(".json") {
        serde_json::from_str::<ConfigFile>(&file_contents).map_err(ConfigFileError::Json)
    } else if file.ends_with(".yaml") {
        serde_yaml::from_str::<ConfigFile>(&file_contents).map_err(ConfigFileError::Yaml)
    } else if file.ends_with(".toml") {
        toml::from_str::<ConfigFile>(&file_contents).map_err(ConfigFileError::Toml)
    } else {
        Err(ConfigFileError::UnsupportedLanguage)
    }?;
    #[cfg(not(unix))]
    if config.server.unix_socket.is_some() {
        use colored::Colorize;
        eprintln!(
            "{}: cannot use unix sockets on non-unix os - ignoring",
            "warning".yellow()
        );
    }

    Ok(config)
}

#[cfg(test)]
mod test {
    use super::*;
    use std::assert_matches;
    use std::io::Write;
    use tempfile::Builder;

    fn check_loaded_file(config: ConfigFile) {
        assert_eq!(config.max_in_flight, Some(8));
        assert_eq!(config.link_prefix, Some("test".into()));
        assert_eq!(config.link_count, Some(8));
        assert_eq!(config.max_depth, Some(MaxDepth(Some(8))));
        assert_matches!(config.force_gzip, Some(true));
        assert_matches!(config.unsafe_allow_html, Some(true));
        assert_eq!(
            config.poison_source.unwrap().as_str(),
            "https://example.com/"
        );

        let metrics = config.metrics.unwrap();

        assert_eq!(metrics.db_path, "miasma.db");
        assert_eq!(metrics.endpoint, Some("/serve-metrics".into()));
        assert_eq!(metrics.username, "admin");
        assert_eq!(metrics.password, "admin-password");
        assert_eq!(config.server.port, Some(8080));
        assert_eq!(config.server.host, Some("127.0.0.1".into()));
        assert_eq!(config.server.unix_socket, Some("miasma.sock".into()));
    }

    #[test]
    fn load_json() {
        let text = r#"{
"server": {
  "port": 8080,
  "host": "127.0.0.1",
  "unix_socket": "miasma.sock"
},
"max_in_flight": 8,
"link_prefix": "test",
"link_count": 8,
"max_depth": 8,
"force_gzip": true,
"unsafe_allow_html": true,
"poison_source": "https://example.com/",
"metrics": {
  "db_path": "miasma.db",
  "username": "admin",
  "password": "admin-password",
  "endpoint": "/serve-metrics"
}
}"#;
        let mut file = Builder::new().suffix(".json").tempfile().unwrap();
        write!(file, "{text}").unwrap();
        let config = load_config_file(file.path().to_str().unwrap()).unwrap();
        check_loaded_file(config);
    }

    #[test]
    fn load_yaml() {
        let text = "\
server:
  port: 8080
  host: 127.0.0.1
  unix_socket: miasma.sock
max_in_flight: 8
link_prefix: test
link_count: 8
max_depth: 8
force_gzip: true
unsafe_allow_html: true
poison_source: https://example.com/
metrics:
  db_path: miasma.db
  username: admin
  password: admin-password
  endpoint: /serve-metrics";
        let mut file = Builder::new().suffix(".yaml").tempfile().unwrap();
        write!(file, "{text}").unwrap();
        let config = load_config_file(file.path().to_str().unwrap()).unwrap();
        check_loaded_file(config);
    }

    #[test]
    fn load_toml() {
        let text = r#"
max_in_flight = 8
link_prefix = "test"
link_count = 8
max_depth = 8
force_gzip = true
unsafe_allow_html = true
poison_source = "https://example.com/"

[server]
port = 8080
host = "127.0.0.1"
unix_socket = "miasma.sock"

[metrics]
db_path = "miasma.db"
username = "admin"
password = "admin-password"
endpoint = "/serve-metrics"
"#;
        let mut file = Builder::new().suffix(".toml").tempfile().unwrap();
        write!(file, "{text}").unwrap();
        let config = load_config_file(file.path().to_str().unwrap()).unwrap();
        check_loaded_file(config);
    }

    #[test]
    fn allows_unset_fields() {
        let text = "
link_count: 5
server:
  port: 0
";
        let mut file = Builder::new().suffix(".yaml").tempfile().unwrap();
        write!(file, "{text}").unwrap();
        let config = load_config_file(file.path().to_str().unwrap()).unwrap();
        assert_eq!(config.link_count, Some(5));
        assert_eq!(config.server.port, Some(0));
        assert_eq!(config.server.host, None);
    }
}
