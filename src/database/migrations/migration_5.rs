use tokio_postgres::Transaction;

use crate::{
    components::telegram::{epsg_3857_to_epsg_4326, filter::StreckenInfoFilter},
    database::DbError,
};

// change filter coordinates from epsg 3857 to epsg 4326
pub async fn migrate(transaction: &Transaction<'_>) -> Result<(), DbError> {
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
            .map(|filter| {
                if let StreckenInfoFilter::Location {
                    mut x,
                    mut y,
                    range,
                } = filter
                {
                    (x, y) = epsg_3857_to_epsg_4326(x, y);
                    StreckenInfoFilter::Location { x, y, range }
                } else {
                    filter
                }
            });
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
