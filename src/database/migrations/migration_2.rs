use crate::database::{DbConnection, DbError};

pub async fn migrate<'a>(connection: &DbConnection<'a>) -> Result<(), DbError> {
    connection
        .execute("ALTER TABLE disruption ADD json JSONB", &[])
        .await?;
    Ok(())
}
