use std::error::Error;
use std::io::Write;
use std::time::Duration;

use colored::Colorize;
use serde::Deserialize;

use miasma::MIASMA_USER_AGENT;

#[derive(Deserialize)]
struct CrateInfo {
    max_stable_version: String,
}

#[derive(Deserialize)]
struct CratesIOResponse {
    #[serde(rename = "crate")]
    crate_info: CrateInfo,
}

const VERSION: &str = env!("CARGO_PKG_VERSION");

pub async fn check_for_new_version() {
    let _res: Result<(), Box<dyn Error>> = async {
        let resp = reqwest::Client::builder()
            .user_agent(MIASMA_USER_AGENT)
            .timeout(Duration::from_secs(5))
            .build()?
            .get("https://crates.io/api/v1/crates/miasma")
            .send()
            .await?
            .json::<CratesIOResponse>()
            .await?;

        let latest = resp.crate_info.max_stable_version;
        print_update_message(VERSION, &latest);

        Ok(())
    }
    .await;

    #[cfg(debug_assertions)]
    #[expect(clippy::used_underscore_binding)]
    if let Err(e) = _res {
        eprintln!("Failed to check for latest version: {e}");
    }
}

fn print_update_message(current: &str, latest: &str) {
    if current != latest {
        // fd redirection problems with eprintln! and gag require writeln! to the live stderr handle
        let mut stderr = std::io::stderr().lock();
        let _ = writeln!(stderr, "\n------- New Version Available -------");
        let _ = writeln!(
            stderr,
            "Installed ({}) -> Latest ({})",
            current.red(),
            latest.green()
        );
        let _ = writeln!(stderr, "-------------------------------------\n");
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use gag::BufferRedirect;
    use serial_test::serial;
    use std::io::Read;

    #[test]
    #[serial]
    fn version_mismatch_prints_to_stderr() {
        let capture = BufferRedirect::stderr().unwrap();
        print_update_message("0.1.0", "0.1.1");
        let _ = std::io::stderr().flush();
        let mut buffer = capture.into_inner();
        let mut output = String::new();
        buffer.read_to_string(&mut output).unwrap();
        assert!(
            output.contains("New Version Available"),
            "expected update banner: {output:?}"
        );
        assert!(
            output.contains("0.1.0"),
            "expected current version: {output:?}"
        );
        assert!(
            output.contains("0.1.1"),
            "expected latest version: {output:?}"
        );
    }

    #[test]
    #[serial]
    fn version_match_prints_nothing_to_stderr() {
        let capture = BufferRedirect::stderr().unwrap();
        print_update_message("0.1.0", "0.1.0");
        let _ = std::io::stderr().flush();
        let mut buffer = capture.into_inner();
        let mut output = String::new();
        buffer.read_to_string(&mut output).unwrap();
        assert!(output.is_empty(), "expected no stderr: {output:?}");
    }
}
