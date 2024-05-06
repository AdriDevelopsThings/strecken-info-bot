use megalodon::megalodon::GetAccountStatusesInputOptions;
use r2d2_sqlite::rusqlite::params;

use crate::{Database, MastodonSender};

use super::ask_are_u_sure;

pub async fn clear_toots(database: Database) {
    if !ask_are_u_sure("Are you sure to delete ALL toots from mastodon created by the authenticated user and clear all toots in the database?
    Please take care of the rate limit so don't use this action to delete a mass of toots") {
        return;
    }
    let client = MastodonSender::create_client_by_env();
    let connection = database.get_connection().unwrap();

    let me = client
        .verify_account_credentials()
        .await
        .expect("Error while verifying account credentials")
        .json;
    let mut options = GetAccountStatusesInputOptions {
        limit: Some(40),
        ..Default::default()
    };

    loop {
        let statuses = client
            .get_account_statuses(me.id.clone(), Some(&options))
            .await
            .expect("Error while fetching statuses from account")
            .json;
        if statuses.is_empty() {
            break;
        }
        options.max_id = Some(statuses.last().unwrap().id.clone());
        for status in statuses {
            println!("Deleting status {}", status.id);
            let delete_response = client.delete_status(status.id).await;
            // This is just a workaround because there is a bug with the megalodon crate
            // The part could be replaced by an unwrap when the issues was fixed
            // https://github.com/h3poteto/megalodon-rs/issues/180
            if let Err(e) = delete_response {
                if let megalodon::error::Error::RequestError(req_err) = e {
                    if req_err.is_decode() {
                        continue;
                    } else {
                        panic!("Error while deleting status: RequestError: {req_err:?}")
                    }
                } else {
                    panic!("Error while deleting status: {e:?}");
                }
            }
        }
    }

    println!("Clearing toots stored in the database");
    connection.execute("DELETE FROM toots", params![]).unwrap();
}
