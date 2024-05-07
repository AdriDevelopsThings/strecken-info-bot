use r2d2_sqlite::rusqlite::params;
use telexide::{
    api::types::SendMessage, client::Context, framework::CommandResult, macros::command,
    model::Message,
};

use super::HashMapDatabase;

#[command(description = "Enable or disable planned disruption updates")]
async fn planned(context: Context, message: Message) -> CommandResult {
    let database = context
        .data
        .write()
        .get::<HashMapDatabase>()
        .unwrap()
        .clone();
    let connection = database.get_connection().unwrap();
    let planned: bool = connection
        .query_row(
            "SELECT show_planned_disruptions FROM user WHERE chat_id=?",
            params![message.chat.get_id()],
            |row| row.get(0),
        )
        .unwrap();
    connection
        .execute(
            "UPDATE user SET show_planned_disruptions=? WHERE chat_id=?",
            params![!planned, message.chat.get_id()],
        )
        .unwrap();
    context.api.send_message(SendMessage::new(
        message.chat.get_id().into(),
        match !planned {
            true => "You will now receive planned disruption updates - run /planned to disable these updates",
            false => "You will no longer receive planned disruption updates - run /planned to reactive them"
        }
    )).await.unwrap();
    Ok(())
}
