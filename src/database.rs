use r2d2::{Pool, PooledConnection};
use r2d2_sqlite::{rusqlite::params, SqliteConnectionManager};

#[derive(Clone)]
pub struct Database {
    connection: Pool<SqliteConnectionManager>,
}

impl Database {
    pub fn new(path: &str) -> Result<Self, r2d2::Error> {
        let manager = SqliteConnectionManager::file(path);
        let pool = Pool::new(manager)?;
        Ok(Self { connection: pool })
    }

    pub fn get_connection(&self) -> Result<PooledConnection<SqliteConnectionManager>, r2d2::Error> {
        self.connection.get()
    }

    pub fn initialize(&self) -> Result<(), r2d2::Error> {
        let connection = self.get_connection()?;
        let user_version: i32 = connection
            .pragma_query_value(None, "user_version", |row| row.get(0))
            .unwrap();
        if user_version <= 0 {
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
            connection.pragma_update(None, "user_version", 1).unwrap();
        }

        if user_version <= 1 {
            // add trigger_warning_list column to user
            connection
                .execute(
                    "ALTER TABLE user
                    ADD trigger_warning_list STRING DEFAULT \"\" NOT NULL;",
                    params![],
                )
                .unwrap();
            connection.pragma_update(None, "user_version", 2).unwrap();
        }
        Ok(())
    }
}
