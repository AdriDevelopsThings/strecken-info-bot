use telexide::{
    api::types::SendMessage,
    client::Context,
    macros::command,
    model::{InlineKeyboardButton, InlineKeyboardMarkup, Message, ReplyMarkup},
    prelude::CommandResult,
};

use super::consts::{ADD_FILTER, CHANGE_FILTER_BEHAVIOUR, REMOVE_FILTER, SHOW_FILTER};

#[command(description = "Filter bearbeiten")]
async fn filter(context: Context, message: Message) -> CommandResult {
    let mut show_btn = InlineKeyboardButton::new("Filter anzeigen", false);
    show_btn.callback_data = Some(SHOW_FILTER.to_string());

    let mut add_btn = InlineKeyboardButton::new("Filter hinzufügen", false);
    add_btn.callback_data = Some(ADD_FILTER.to_string());

    let mut remove_btn = InlineKeyboardButton::new("Filter entfernen", false);
    remove_btn.callback_data = Some(REMOVE_FILTER.to_string());

    let mut change_behaviour_btn = InlineKeyboardButton::new("Filterverhalten ändern", false);
    change_behaviour_btn.callback_data = Some(CHANGE_FILTER_BEHAVIOUR.to_string());

    let mut markup = InlineKeyboardMarkup::new();

    markup.add_row(vec![show_btn]);
    markup.add_row(vec![add_btn]);
    markup.add_row(vec![remove_btn]);
    markup.add_row(vec![change_behaviour_btn]);
    let mut send_message = SendMessage::new(
        message.chat.get_id().into(),
        "Dich nerven die vielen Nachrichten? Zeit einen Filter anzulegen, um nur noch Nachrichten zu erhalten, die dich interessieren. Eine Störung wird dir dann nur noch zugeschickt, wenn die eingestellten Filter auf die Störung zutreffen. Ob alle oder nur ein Filter zutreffen müssen kannst du im Filterverhalten einstellen. Was möchtest du nun tun?\nAchtung: Aktuell ist das Feature noch sehr experimentell, für Bugs oder Anregungen: /feedback ".to_string(),
    );
    send_message.reply_markup = Some(ReplyMarkup::InlineKeyboardMarkup(markup));
    context.api.send_message(send_message).await?;
    Ok(())
}
