use tokio_postgres::Transaction;

use crate::database::DbError;

pub async fn migrate(transaction: &Transaction<'_>) -> Result<(), DbError> {
    transaction
        .execute(
            "ALTER TABLE telegram_user ADD one_filter_enough BOOLEAN DEFAULT FALSE NOT NULL",
            &[],
        )
        .await?;
    Ok(())
}
