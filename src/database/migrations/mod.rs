use tracing::info;

use super::DbConnection;

mod migration_1;
mod migration_2;
mod migration_3;
mod migration_4;
mod migration_5;
mod migration_6;

const MAX_MIGRATION: i16 = 6;

pub async fn run_migrations(mut connection: DbConnection<'_>) {
    connection
        .execute(
            "CREATE TABLE IF NOT EXISTS migration(id SMALLINT NOT NULL PRIMARY KEY)",
            &[],
        )
        .await
        .expect("Error while creating migration table");

    for migration_number in 1_i16..MAX_MIGRATION + 1 {
        if connection
            .query_opt("SELECT id FROM migration WHERE id=$1", &[&migration_number])
            .await
            .expect("Error while checking migration")
            .is_some()
        {
            continue;
        }
        info!("Running database migration {migration_number}");

        let transaction = connection
            .transaction()
            .await
            .expect("Error while creating transaction");

        match migration_number {
            1 => migration_1::migrate(&transaction).await,
            2 => migration_2::migrate(&transaction).await,
            3 => migration_3::migrate(&transaction).await,
            4 => migration_4::migrate(&transaction).await,
            5 => migration_5::migrate(&transaction).await,
            6 => migration_6::migrate(&transaction).await,
            _ => unreachable!(),
        }
        .expect("Error while running migration");

        transaction
            .execute("INSERT INTO migration(id) VALUES($1)", &[&migration_number])
            .await
            .expect("Error while updating migration");

        transaction
            .commit()
            .await
            .expect("Error while commiting transaction");
    }
}
