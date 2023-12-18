use r2d2::PooledConnection;
use r2d2_sqlite::{rusqlite::params, SqliteConnectionManager};

/// add start_time and end_time datetimes to database
pub fn migrate(connection: &PooledConnection<SqliteConnectionManager>) {
    connection
        .execute(
            "ALTER TABLE disruption
                ADD start_time DATETIME",
            params![],
        )
        .unwrap();
    connection
        .execute(
            "ALTER TABLE disruption
                ADD end_time DATETIME",
            params![],
        )
        .unwrap();
}
