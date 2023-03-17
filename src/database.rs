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

        Ok(())
    }
}
