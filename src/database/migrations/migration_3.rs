use tokio_postgres::Transaction;

use crate::database::DbError;

pub async fn migrate(transaction: &Transaction<'_>) -> Result<(), DbError> {
    transaction
        .execute(
            "ALTER TABLE telegram_user ADD filters JSONB[] DEFAULT array[]::JSONB[] NOT NULL",
            &[],
        )
        .await?;
    Ok(())
}
