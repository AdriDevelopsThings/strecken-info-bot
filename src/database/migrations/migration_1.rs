use r2d2::PooledConnection;
use r2d2_sqlite::{rusqlite::params, SqliteConnectionManager};

/// initial database migration
pub fn migrate(connection: &PooledConnection<SqliteConnectionManager>) {
    connection
        .execute(
            "CREATE TABLE IF NOT EXISTS user (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                chat_id INTEGER NOT NULL UNIQUE
            )",
            params![],
        )
        .unwrap();

    connection
        .execute(
            "CREATE TABLE IF NOT EXISTS disruption (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                him_id STRING NOT NULL UNIQUE,
                hash STRING NOT NULL
            )",
            params![],
        )
        .unwrap();
}
