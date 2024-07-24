use crate::database::{DbConnection, DbError};

pub async fn migrate<'a>(connection: &DbConnection<'a>) -> Result<(), DbError> {
    connection
        .execute(
            "ALTER TABLE telegram_user ADD one_filter_enough BOOLEAN DEFAULT FALSE NOT NULL",
            &[],
        )
        .await?;
    Ok(())
}
