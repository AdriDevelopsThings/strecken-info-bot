use r2d2::PooledConnection;
use r2d2_sqlite::{rusqlite::params, SqliteConnectionManager};

/// add show_planned_disruptions column to user
pub fn migrate(connection: &PooledConnection<SqliteConnectionManager>) {
    connection
        .execute(
            "ALTER TABLE user
                    ADD show_planned_disruptions INTEGER DEFAULT 0 NOT NULL;",
            params![],
        )
        .unwrap();
}
