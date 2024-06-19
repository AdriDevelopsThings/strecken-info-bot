use r2d2::PooledConnection;
use r2d2_sqlite::{rusqlite::params, SqliteConnectionManager};

/// rename 'him_id' to 'key'
pub fn migrate(connection: &PooledConnection<SqliteConnectionManager>) {
    connection
        .execute(
            "ALTER TABLE disruption
        RENAME COLUMN him_id TO key",
            params![],
        )
        .unwrap();
}
