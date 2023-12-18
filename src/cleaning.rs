use std::time::Duration;

use log::debug;
use r2d2_sqlite::rusqlite::params;
use tokio::time::sleep;

use crate::Database;

pub fn start_cleaning(database: Database) {
    tokio::spawn(async move {
        loop {
            {
                let connection = database.get_connection().unwrap();
                let affected = connection
                    .execute(
                        "DELETE FROM disruption WHERE end_time < date('now', '-2 day')",
                        params![],
                    )
                    .unwrap();
                if affected > 0 {
                    debug!("Cleaned {affected} old disruptions from database");
                }
            }

            sleep(Duration::from_secs(60 * 30)).await;
        }
    });
}
