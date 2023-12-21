use r2d2_sqlite::rusqlite::params;

use crate::{cli::ask_are_u_sure, Database};

pub async fn reset_disruptions(database: Database) {
    let connection = database.get_connection().unwrap();
    if !ask_are_u_sure("Are you sure to delete all saved disruptions? Lots of new updates will be sent after this?") {
        return;
    }
    connection
        .execute("DELETE FROM disruption", params![])
        .unwrap();
    println!("All saved disruptions removed");
}
