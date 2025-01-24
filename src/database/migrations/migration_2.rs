use crate::database::{DbConnection, DbError};

pub async fn migrate(connection: &DbConnection<'_>) -> Result<(), DbError> {
    connection
        .execute("ALTER TABLE disruption ADD json JSONB", &[])
        .await?;
    Ok(())
}
