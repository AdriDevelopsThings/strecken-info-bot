use tokio_postgres::{types::ToSql, Transaction};

use crate::{
    components::telegram::filter::{Filter, StreckenInfoFilter},
    data::strecken_info::STRECKEN_INFO_TYPE,
    database::DbError,
};

async fn migrate_telegram_filters(transaction: &Transaction<'_>) -> Result<(), DbError> {
    let rows = transaction
        .query(
            "SELECT id, filters FROM telegram_user WHERE array_length(filters, 1) > 0",
            &[],
        )
        .await?;
    for row in rows {
        let id: i32 = row.get(0);
        let filters = row
            .get::<_, Vec<serde_json::Value>>(1)
            .into_iter()
            .map(|filter| serde_json::from_value::<StreckenInfoFilter>(filter).unwrap())
            .map(Filter::StreckenInfo);
        transaction
            .execute(
                "UPDATE telegram_user SET filters=$1 WHERE id=$2",
                &[
                    &filters
                        .map(|filter| serde_json::to_value(filter).unwrap())
                        .collect::<Vec<serde_json::Value>>(),
                    &id,
                ],
            )
            .await?;
    }

    Ok(())
}

async fn migrate_disruptions_table(transaction: &Transaction<'_>) -> Result<(), DbError> {
    let statements: &[(&str, &[&(dyn ToSql + Sync)])] = &[
        ("ALTER TABLE disruption DROP COLUMN hash", &[]),
        ("ALTER TABLE disruption ADD COLUMN data_source TEXT", &[]),
        (
            "UPDATE disruption SET data_source=$1",
            &[&STRECKEN_INFO_TYPE],
        ),
        (
            "ALTER TABLE disruption ALTER COLUMN data_source SET NOT NULL",
            &[],
        ),
        (
            "ALTER TABLE disruption DROP CONSTRAINT disruption_key_key",
            &[],
        ),
        ("ALTER TABLE disruption ADD CONSTRAINT disruption_data_source_key_key UNIQUE (key, data_source)", &[])
    ];

    for statement in statements {
        transaction.execute(statement.0, statement.1).await?;
    }
    Ok(())
}

pub async fn migrate(transaction: &Transaction<'_>) -> Result<(), DbError> {
    migrate_telegram_filters(transaction).await?;
    migrate_disruptions_table(transaction).await?;
    Ok(())
}
