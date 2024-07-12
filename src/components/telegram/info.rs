use std::{env, option_env};
use telexide::{api::types::SendMessage, prelude::*};

#[command(description = "Aktuelle Bot Version")]
async fn version(context: Context, message: Message) -> CommandResult {
    let commit_hash = option_env!("GIT_SHA")
        .map(|sha| format!(" (Commit: {sha})"))
        .unwrap_or_default();
    context
        .api
        .send_message(SendMessage::new(
            message.chat.get_id().into(),
            format!(
                "Dieser Bot läuft auf Version {}{}",
                env!("CARGO_PKG_VERSION"),
                commit_hash
            ),
        ))
        .await?;
    Ok(())
}

#[command(description = "Link zum GitHub Repository")]
async fn git(context: Context, message: Message) -> CommandResult {
    context
        .api
        .send_message(SendMessage::new(
            message.chat.get_id().into(),
            format!(
                "Schau dir hier den Sourcecode an: {}",
                env!("CARGO_PKG_REPOSITORY")
            ),
        ))
        .await?;
    Ok(())
}

#[command(description = "Feedback oder Bugs")]
async fn feedback(context: Context, message: Message) -> CommandResult {
    context
        .api
        .send_message(SendMessage::new(
            message.chat.get_id().into(),
            format!(
                "Erstelle einfach ein Issue im GitHub Repository: {}{}",
                env!("CARGO_PKG_REPOSITORY"),
                match option_env!("CARGO_PKG_AUTHORS") {
                    Some(authors) => format!("\nOder kontaktiere {authors}"),
                    None => String::new(),
                }
            ),
        ))
        .await?;
    Ok(())
}
