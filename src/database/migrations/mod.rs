use log::info;
use r2d2::PooledConnection;
use r2d2_sqlite::SqliteConnectionManager;

mod migration_1;
mod migration_2;
mod migration_3;
mod migration_4;
mod migration_5;

type MigrationFunction = fn(&PooledConnection<SqliteConnectionManager>);

static MIGRATIONS: &[(i32, MigrationFunction)] = &[
    (1, migration_1::migrate),
    (2, migration_2::migrate),
    (3, migration_3::migrate),
    (4, migration_4::migrate),
    (5, migration_5::migrate),
];

fn run_migration(
    connection: &PooledConnection<SqliteConnectionManager>,
    migration_number: i32,
    migration_function: MigrationFunction,
) {
    let user_version: i32 = connection
        .pragma_query_value(None, "user_version", |row| row.get(0))
        .unwrap();
    if user_version < migration_number {
        if migration_number - user_version > 1 {
            panic!("Invalid database migration sequence, this migration could break your database");
        }
        info!("Running database migration {migration_number}");
        migration_function(connection);
        connection
            .pragma_update(None, "user_version", migration_number)
            .unwrap();
    }
}

pub fn run_migrations(connection: &PooledConnection<SqliteConnectionManager>) {
    for (migration_number, migration_function) in MIGRATIONS.iter() {
        run_migration(connection, *migration_number, *migration_function);
    }
}
