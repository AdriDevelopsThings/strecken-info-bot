use tokio_postgres::Transaction;

use crate::database::DbError;

pub async fn migrate(tranaction: &Transaction<'_>) -> Result<(), DbError> {
    tranaction
        .execute("ALTER TABLE disruption ADD json JSONB", &[])
        .await?;
    Ok(())
}
