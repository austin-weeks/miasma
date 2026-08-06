use std::{collections::HashMap, mem};

use colored::Colorize;
use diesel::{prelude::*, upsert::excluded};

use crate::metrics::{RESULTS_PER_PAGE, migrations};

use super::MetricsError;

#[allow(clippy::wildcard_imports)]
use self::user_agents::dsl::*;

diesel::table! {
    user_agents (agent) {
        agent -> Text,
        request_count -> BigInt,
        poison_bytes_sent -> BigInt,
        total_bytes_sent -> BigInt,
    }
}

#[derive(Clone)]
pub struct UserAgent(String);
impl UserAgent {
    pub fn new(user_agent: &str) -> Self {
        // Truncate the user agent string to ensure we don't store massive values.
        // There's a very small chance that scrapers have big ole user agents and
        // might try to exploit the fact that we're storing them.
        let truncated_user_agent = user_agent
            .chars()
            .take(Metrics::MAX_USER_AGENT_CHAR_LENGTH)
            .collect();
        Self(truncated_user_agent)
    }
    fn inner(&self) -> &str {
        self.0.as_ref()
    }
}

pub struct Metrics {
    counts: HashMap<String, MetricsEntry>,
    unflushed_count: u32,
    db_path: String,
}

#[derive(Selectable, Queryable)]
#[diesel(table_name = user_agents)]
#[cfg_attr(test, derive(Clone))]
pub struct MetricsEntry {
    pub request_count: i64,
    pub poison_bytes_sent: i64,
    pub total_bytes_sent: i64,
}

impl Metrics {
    const MAX_UNFLUSHED_COUNT: u32 = 1_000;
    const MAX_USER_AGENT_CHAR_LENGTH: usize = 1024;

    pub fn new(db_path: String) -> Result<Self, MetricsError> {
        let mut conn = SqliteConnection::establish(&db_path)?;
        migrations::run_db_migrations(&mut conn)?;

        Ok(Self {
            db_path,
            counts: HashMap::new(),
            unflushed_count: 0,
        })
    }

    /// Increment the request count for the supplied user agent by one.
    pub fn count_request(&mut self, user_agent: &UserAgent) {
        self.unflushed_count += 1;
        self.entry(user_agent).request_count += 1;
        self.flush_if_full();
    }

    pub fn record_poison_bytes(&mut self, user_agent: &UserAgent, bytes_sent: usize) {
        self.entry(user_agent).poison_bytes_sent += i64::try_from(bytes_sent).unwrap_or(i64::MAX);
        self.flush_if_full();
    }

    pub fn record_total_bytes(&mut self, user_agent: &UserAgent, bytes_sent: usize) {
        self.entry(user_agent).total_bytes_sent += i64::try_from(bytes_sent).unwrap_or(i64::MAX);
        self.flush_if_full();
    }

    fn entry(&mut self, user_agent: &UserAgent) -> &mut MetricsEntry {
        if !self.counts.contains_key(user_agent.inner()) {
            self.counts.insert(
                user_agent.inner().to_owned(),
                MetricsEntry {
                    request_count: 0,
                    poison_bytes_sent: 0,
                    total_bytes_sent: 0,
                },
            );
        }
        self.counts
            .get_mut(user_agent.inner())
            .expect("we just inserted this user agent if not present")
    }

    /// Flush metrics to the database in a non-blocking background task if full.
    pub fn flush_if_full(&mut self) {
        if self.unflushed_count < Metrics::MAX_UNFLUSHED_COUNT {
            return;
        }
        self.unflushed_count = 0;
        let flushing = mem::take(&mut self.counts);
        let db_path = self.db_path.clone();

        tokio::task::spawn_blocking(move || {
            flush_to_db(flushing, &db_path);
        });
    }

    /// Flush metrics to the database and block until completion.
    pub fn flush_blocking(&mut self) {
        let flushing = mem::take(&mut self.counts);

        flush_to_db(flushing, &self.db_path);
    }

    /// List a portion of entries in the metrics database by request count.
    pub fn list_useragents_by_count(
        // TODO: add ordering param (order by requests, poison sent, etc.)
        &mut self,
        page: u32,
    ) -> Result<Vec<(String, MetricsEntry)>, MetricsError> {
        let offset = page.saturating_sub(1) * RESULTS_PER_PAGE;
        let mut conn = SqliteConnection::establish(&self.db_path)?;
        let entries = user_agents
            .select((agent, MetricsEntry::as_select()))
            .order_by(request_count.desc())
            .limit(RESULTS_PER_PAGE as i64)
            .offset(offset as i64)
            .load(&mut conn)?;
        Ok(entries)
    }
}

fn flush_to_db(counts: HashMap<String, MetricsEntry>, db_path: &str) {
    let mut conn = match SqliteConnection::establish(db_path) {
        Ok(c) => c,
        Err(e) => {
            eprintln!("{}: {e}", "Failed to connect to metrics database".red());
            return;
        }
    };
    #[allow(unused_must_use)]
    diesel::sql_query("PRAGMA busy_timeout = 5000").execute(&mut conn);

    let rows = counts
        .into_iter()
        .map(|(ua, row)| {
            (
                agent.eq(ua),
                request_count.eq(row.request_count),
                poison_bytes_sent.eq(row.poison_bytes_sent),
                total_bytes_sent.eq(row.total_bytes_sent),
            )
        })
        .collect::<Vec<_>>();

    if let Err(e) = diesel::insert_into(user_agents)
        .values(rows)
        .on_conflict(agent)
        .do_update()
        .set((
            request_count.eq(request_count + excluded(request_count)),
            poison_bytes_sent.eq(poison_bytes_sent + excluded(poison_bytes_sent)),
            total_bytes_sent.eq(total_bytes_sent + excluded(total_bytes_sent)),
        ))
        .execute(&mut conn)
    {
        eprintln!("{}: {e}", "Failed to write metrics to database".red());
    }
}

// Ensure metrics are flushed when going out of scope.
impl Drop for Metrics {
    fn drop(&mut self) {
        self.flush_blocking();
    }
}

#[cfg(test)]
mod test {
    use crate::test_utils;

    use super::*;

    #[test]
    fn recording_methods_track_per_agent_metrics() {
        let mut expected_request_count = 0;
        let mut expected_poison_bytes_sent = 0;
        let mut expected_total_bytes_sent = 0;
        macro_rules! assert_expected_entry {
            ($entry:expr) => {
                assert_eq!($entry.request_count, expected_request_count);
                assert_eq!($entry.poison_bytes_sent, expected_poison_bytes_sent);
                assert_eq!($entry.total_bytes_sent, expected_total_bytes_sent);
            };
        }

        let (_file, path) = test_utils::temp_file();
        let mut metrics = Metrics::new(path).unwrap();
        let user_agent = UserAgent::new("bot");

        expected_request_count += 1;
        metrics.count_request(&user_agent);
        assert_expected_entry!(metrics.counts.get("bot").unwrap());

        expected_poison_bytes_sent += 64;
        metrics.record_poison_bytes(&user_agent, 64);
        assert_expected_entry!(metrics.counts.get("bot").unwrap());

        expected_total_bytes_sent += 1000;
        metrics.record_total_bytes(&user_agent, 1000);
        assert_expected_entry!(metrics.counts.get("bot").unwrap());

        expected_poison_bytes_sent += 128;
        metrics.record_poison_bytes(&user_agent, 128);
        assert_expected_entry!(metrics.counts.get("bot").unwrap());

        expected_request_count += 1;
        metrics.count_request(&user_agent);
        assert_expected_entry!(metrics.counts.get("bot").unwrap());
    }

    #[test]
    fn data_persisted_on_flush() {
        let (_file, db_path) = test_utils::temp_file();
        let mut conn = SqliteConnection::establish(&db_path).expect("failed to connect to test db");
        diesel::sql_query(
            "CREATE TABLE user_agents (
                agent TEXT PRIMARY KEY,
                request_count INTEGER NOT NULL,
                poison_bytes_sent INTEGER NOT NULL,
                total_bytes_sent INTEGER NOT NULL
            )",
        )
        .execute(&mut conn)
        .expect("failed to create test table");

        let expected = [
            (
                "miasma/0.1".to_owned(),
                MetricsEntry {
                    request_count: 5,
                    poison_bytes_sent: 64,
                    total_bytes_sent: 128,
                },
            ),
            (
                "claudebot".to_owned(),
                MetricsEntry {
                    request_count: 10,
                    poison_bytes_sent: 10_000,
                    total_bytes_sent: 12_000,
                },
            ),
            (
                "safari".to_owned(),
                MetricsEntry {
                    request_count: 15,
                    poison_bytes_sent: 2048,
                    total_bytes_sent: 40_000,
                },
            ),
        ];

        flush_to_db(HashMap::from(expected.clone()), &db_path);

        let mut conn =
            SqliteConnection::establish(&db_path).expect("failed to connect to database");

        let rows = user_agents
            .select((agent, MetricsEntry::as_select()))
            .load::<(String, MetricsEntry)>(&mut conn)
            .expect("failed to query test db");

        assert_eq!(rows.len(), expected.len());
        for (expected_ua, expected_row) in expected {
            let (actual_ua, actual_row) = rows
                .iter()
                .find(|(ua, _)| ua.as_str() == expected_ua)
                .expect("expected row not found in test db");
            assert_eq!(actual_ua, &expected_ua);
            assert_eq!(actual_row.request_count, expected_row.request_count);
            assert_eq!(actual_row.poison_bytes_sent, expected_row.poison_bytes_sent);
            assert_eq!(actual_row.total_bytes_sent, expected_row.total_bytes_sent);
        }
    }
}
