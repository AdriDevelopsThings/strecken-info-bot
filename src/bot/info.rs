use std::{env, option_env};
use telexide::{api::types::SendMessage, prelude::*};

#[command(description = "Get the current version of this bot")]
async fn version(context: Context, message: Message) -> CommandResult {
    let commit_hash = option_env!("GIT_SHA")
        .map(|sha| format!(" (Commit: {sha})"))
        .unwrap_or_default();
    context
        .api
        .send_message(SendMessage::new(
            message.chat.get_id().into(),
            format!(
                "The bot is running on version {}{}",
                env!("CARGO_PKG_VERSION"),
                commit_hash
            ),
        ))
        .await?;
    Ok(())
}

#[command(description = "Get a link to the git repository")]
async fn git(context: Context, message: Message) -> CommandResult {
    context
        .api
        .send_message(SendMessage::new(
            message.chat.get_id().into(),
            format!(
                "You can take a look to our sourcecode here: {}",
                env!("CARGO_PKG_REPOSITORY")
            ),
        ))
        .await?;
    Ok(())
}
