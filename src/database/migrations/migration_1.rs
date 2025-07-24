use tokio_postgres::Transaction;

use crate::database::DbError;

/// initial database migration
pub async fn migrate(transaction: &Transaction<'_>) -> Result<(), DbError> {
    transaction
        .execute(
            "CREATE TABLE telegram_user (
                    id SERIAL PRIMARY KEY,
                    chat_id BIGINT NOT NULL UNIQUE,
                    trigger_warnings TEXT[] DEFAULT array[]::varchar[] NOT NULL,
                    show_planned_disruptions BOOLEAN DEFAULT FALSE NOT NULL
                )",
            &[],
        )
        .await?;

    transaction
        .execute(
            "CREATE TABLE disruption (
                id SERIAL PRIMARY KEY,
                key TEXT NOT NULL UNIQUE,
                hash VARCHAR(32) NOT NULL,
                start_time TIMESTAMP WITHOUT TIME ZONE,
                end_time TIMESTAMP WITHOUT TIME ZONE
            )",
            &[],
        )
        .await?;

    transaction
        .execute(
            "CREATE TABLE mastodon_toot (
        id SERIAL PRIMARY KEY,
        disruption_id INTEGER NOT NULL,
        status_id VARCHAR(255),
        CONSTRAINT disruption_id
            FOREIGN KEY (disruption_id)
                REFERENCES disruption(id) ON DELETE CASCADE
    )",
            &[],
        )
        .await?;

    Ok(())
}
