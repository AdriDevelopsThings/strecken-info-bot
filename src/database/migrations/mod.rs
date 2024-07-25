use log::info;

use super::DbConnection;

mod migration_1;
mod migration_2;
mod migration_3;
mod migration_4;
mod migration_5;

pub async fn run_migrations(connection: DbConnection<'_>) {
    connection
        .execute(
            "CREATE TABLE IF NOT EXISTS migration(id SMALLINT NOT NULL PRIMARY KEY)",
            &[],
        )
        .await
        .expect("Error while creating migration table");

    for migration_number in 1_i16..6_i16 {
        if connection
            .query_opt("SELECT id FROM migration WHERE id=$1", &[&migration_number])
            .await
            .expect("Error while checking migration")
            .is_some()
        {
            continue;
        }
        info!("Running database migration {migration_number}");
        match migration_number {
            1 => migration_1::migrate(&connection).await,
            2 => migration_2::migrate(&connection).await,
            3 => migration_3::migrate(&connection).await,
            4 => migration_4::migrate(&connection).await,
            5 => migration_5::migrate(&connection).await,
            _ => unreachable!(),
        }
        .expect("Error while running migration");

        connection
            .execute("INSERT INTO migration(id) VALUES($1)", &[&migration_number])
            .await
            .expect("Error while updating migration");
    }
}
