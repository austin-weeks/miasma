use diesel::{
    Connection, RunQueryDsl, SqliteConnection, deserialize::QueryableByName, sql_types::Integer,
};

use crate::metrics::MetricsError;

const LATEST_VERSION: i32 = 2;

pub fn run_db_migrations(conn: &mut SqliteConnection) -> Result<(), MetricsError> {
    // Run the migrations within a transaction so we don't have to
    // save and fetch the user_version after every step.
    conn.transaction(migrations_impl)
}

fn migrations_impl(conn: &mut SqliteConnection) -> Result<(), MetricsError> {
    let user_version = query_user_version(conn)?;
    if user_version == LATEST_VERSION {
        return Ok(());
    }

    if user_version < 1 {
        // We never set user_version = 1 on the initial table creation, so create if not exists,
        // then run the rest of the migrations...
        //
        // When we eventually release v1.0.0 we can update this to just create all tables cleanly
        // if user_version is 0 rather than running all the migrations sequentially. For now we'll
        // do it this way to make sure it's backwards-compatible.
        diesel::sql_query(
            "CREATE TABLE IF NOT EXISTS user_agents (
                agent TEXT PRIMARY KEY,
                count INTEGER NOT NULL
            )",
        )
        .execute(conn)?;
    }
    if user_version < 2 {
        diesel::sql_query("ALTER TABLE user_agents RENAME COLUMN count TO request_count")
            .execute(conn)?;
        diesel::sql_query(
            "ALTER TABLE user_agents ADD COLUMN poison_bytes_sent INTEGER NOT NULL DEFAULT 0",
        )
        .execute(conn)?;
        diesel::sql_query(
            "ALTER TABLE user_agents ADD COLUMN total_bytes_sent INTEGER NOT NULL DEFAULT 0",
        )
        .execute(conn)?;
    }

    diesel::sql_query(format!("PRAGMA user_version = {LATEST_VERSION}")).execute(conn)?;

    Ok(())
}

fn query_user_version(conn: &mut SqliteConnection) -> Result<i32, MetricsError> {
    #[derive(QueryableByName)]
    struct UserVersion {
        #[diesel(sql_type = Integer)]
        user_version: i32,
    }
    let res = diesel::sql_query("PRAGMA user_version").get_result::<UserVersion>(conn)?;
    Ok(res.user_version)
}

#[cfg(test)]
mod test {
    use crate::test_utils;

    use super::*;
    use diesel::sql_types::Text;

    #[derive(QueryableByName)]
    struct ColumnName {
        #[diesel(sql_type = Text)]
        name: String,
    }

    fn query_column_names(conn: &mut SqliteConnection) -> Vec<String> {
        diesel::sql_query("SELECT name FROM pragma_table_info('user_agents')")
            .load::<ColumnName>(conn)
            .unwrap()
            .into_iter()
            .map(|c| c.name)
            .collect()
    }

    #[test]
    fn fresh_db_creates_latest_schema() {
        let (_file, path) = test_utils::temp_file();
        let mut conn = SqliteConnection::establish(&path).unwrap();

        run_db_migrations(&mut conn).unwrap();

        assert_eq!(query_user_version(&mut conn).unwrap(), LATEST_VERSION);
        assert_eq!(
            query_column_names(&mut conn),
            vec![
                "agent",
                "request_count",
                "poison_bytes_sent",
                "total_bytes_sent",
            ]
        );
    }

    #[test]
    fn upgrades_v1_schema_and_preserves_data() {
        #[derive(QueryableByName)]
        struct Row {
            #[diesel(sql_type = Text)]
            agent: String,
            #[diesel(sql_type = Integer)]
            request_count: i32,
            #[diesel(sql_type = Integer)]
            poison_bytes_sent: i32,
            #[diesel(sql_type = Integer)]
            total_bytes_sent: i32,
        }

        let (_file, path) = test_utils::temp_file();
        let mut conn = SqliteConnection::establish(&path).unwrap();

        diesel::sql_query(
            "CREATE TABLE user_agents (
                agent TEXT PRIMARY KEY,
                count INTEGER NOT NULL
            )",
        )
        .execute(&mut conn)
        .unwrap();
        diesel::sql_query("INSERT INTO user_agents (agent, count) VALUES ('claudebot', 10)")
            .execute(&mut conn)
            .unwrap();

        run_db_migrations(&mut conn).unwrap();

        assert_eq!(query_user_version(&mut conn).unwrap(), LATEST_VERSION);
        assert_eq!(
            query_column_names(&mut conn),
            vec![
                "agent",
                "request_count",
                "poison_bytes_sent",
                "total_bytes_sent",
            ]
        );

        let rows = diesel::sql_query("SELECT * FROM user_agents")
            .load::<Row>(&mut conn)
            .unwrap();
        assert_eq!(rows.len(), 1);
        let row = &rows[0];
        assert_eq!(row.agent, "claudebot");
        assert_eq!(row.request_count, 10);
        assert_eq!(row.poison_bytes_sent, 0);
        assert_eq!(row.total_bytes_sent, 0);
    }

    #[test]
    fn migrations_are_idempotent() {
        let (_file, path) = test_utils::temp_file();
        let mut conn = SqliteConnection::establish(&path).unwrap();

        run_db_migrations(&mut conn).unwrap();
        run_db_migrations(&mut conn).unwrap();

        assert_eq!(query_user_version(&mut conn).unwrap(), LATEST_VERSION);
        assert_eq!(
            query_column_names(&mut conn),
            vec![
                "agent",
                "request_count",
                "poison_bytes_sent",
                "total_bytes_sent",
            ]
        );
    }
}
