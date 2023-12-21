use r2d2::PooledConnection;
use r2d2_sqlite::{rusqlite::params, SqliteConnectionManager};

/// add mastodon function
pub fn migrate(connection: &PooledConnection<SqliteConnectionManager>) {
    connection
        .execute(
            "CREATE TABLE toots (
        id INTEGER PRIMARY KEY AUTOINCREMENT,
        disruption_id INTEGER,
        status_id VARCHAR(255),
        FOREIGN KEY (disruption_id) REFERENCES disruption(id) ON DELETE CASCADE
    )",
            params![],
        )
        .unwrap();
}
