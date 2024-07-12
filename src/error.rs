use thiserror::Error;

use crate::database::DbError;

#[derive(Error, Debug)]
pub enum StreckenInfoBotError {
    #[error("database error")]
    DatabaseError(#[from] DbError),
    #[error("serde json (de-)serialization error")]
    SerdeError(#[from] serde_json::Error),
}

impl From<bb8_postgres::tokio_postgres::Error> for StreckenInfoBotError {
    fn from(value: bb8_postgres::tokio_postgres::Error) -> Self {
        Self::DatabaseError(value.into())
    }
}
