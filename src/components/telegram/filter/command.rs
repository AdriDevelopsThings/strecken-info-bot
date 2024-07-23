use telexide::{
    api::types::SendMessage,
    client::Context,
    macros::command,
    model::{InlineKeyboardButton, InlineKeyboardMarkup, Message, ReplyMarkup},
    prelude::CommandResult,
};

use super::consts::{ADD_FILTER, REMOVE_FILTER, SHOW_FILTER};

#[command(description = "Filter bearbeiten")]
async fn filter(context: Context, message: Message) -> CommandResult {
    let mut show_btn = InlineKeyboardButton::new("Filter anzeigen", false);
    show_btn.callback_data = Some(SHOW_FILTER.to_string());

    let mut add_btn = InlineKeyboardButton::new("Filter hinzufügen", false);
    add_btn.callback_data = Some(ADD_FILTER.to_string());

    let mut remove_btn = InlineKeyboardButton::new("Filter entfernen", false);
    remove_btn.callback_data = Some(REMOVE_FILTER.to_string());

    let mut markup = InlineKeyboardMarkup::new();

    markup.add_button(show_btn);
    markup.add_button(add_btn);
    markup.add_button(remove_btn);
    let mut send_message = SendMessage::new(
        message.chat.get_id().into(),
        "Dich nerven die vielen Nachrichten? Zeit einen Filter anzulegen, um nur noch Nachrichten zu erhalten, die dich interessieren. Eine Störung wird dir dann nur noch zugeschickt, wenn alle eingestellten Filter auf die Störung zutreffen. Was möchtest du nun tun?\nAchtung: Aktuell ist das Feature noch sehr experimentell, für Bugs oder Anregungen: /feedback ".to_string(),
    );
    send_message.reply_markup = Some(ReplyMarkup::InlineKeyboardMarkup(markup));
    context.api.send_message(send_message).await?;
    Ok(())
}
