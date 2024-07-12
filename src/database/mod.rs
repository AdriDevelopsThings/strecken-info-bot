use bb8::{Pool, PooledConnection};
use bb8_postgres::{tokio_postgres::NoTls, PostgresConnectionManager};
use thiserror::Error;

use self::migrations::run_migrations;

mod migrations;

pub type DbConnection<'a> = PooledConnection<'a, PostgresConnectionManager<NoTls>>;
#[derive(Error, Debug)]
pub enum DbError {
    #[error("pool error")]
    PoolError(#[from] bb8::RunError<bb8_postgres::tokio_postgres::Error>),
    #[error("postgres error")]
    PostgresError(#[from] bb8_postgres::tokio_postgres::Error),
}

#[derive(Clone)]
pub struct Database {
    connection: Pool<PostgresConnectionManager<NoTls>>,
}

impl Database {
    pub async fn new(config: &str) -> Result<Self, DbError> {
        let manager = PostgresConnectionManager::new(
            config.parse().expect("Invalid postgresql config"),
            NoTls,
        );
        let pool = Pool::builder().build(manager).await?;
        Ok(Self { connection: pool })
    }

    pub async fn get_connection(&self) -> Result<DbConnection<'_>, DbError> {
        Ok(self.connection.get().await?)
    }

    pub async fn initialize(&self) -> Result<(), DbError> {
        let connection = self.get_connection().await?;
        run_migrations(connection).await;
        Ok(())
    }
}
