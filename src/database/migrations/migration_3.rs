use crate::database::{DbConnection, DbError};

pub async fn migrate(connection: &DbConnection<'_>) -> Result<(), DbError> {
    connection
        .execute(
            "ALTER TABLE telegram_user ADD filters JSONB[] DEFAULT array[]::JSONB[] NOT NULL",
            &[],
        )
        .await?;
    Ok(())
}
