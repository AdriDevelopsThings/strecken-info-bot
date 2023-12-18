use r2d2::PooledConnection;
use r2d2_sqlite::{rusqlite::params, SqliteConnectionManager};

/// add trigger_warning_list column to user
pub fn migrate(connection: &PooledConnection<SqliteConnectionManager>) {
    connection
        .execute(
            "ALTER TABLE user
                    ADD trigger_warning_list STRING DEFAULT \"\" NOT NULL;",
            params![],
        )
        .unwrap();
    connection.pragma_update(None, "user_version", 2).unwrap();
}
