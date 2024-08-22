use std::error::Error;

use log::warn;
use telexide::{
    api::types::{AnswerCallbackQuery, SendMessage},
    client::Context,
    macros::prepare_listener,
    model::{
        CallbackQuery, InlineKeyboardButton, InlineKeyboardMarkup, Message, MessageContent,
        ReplyMarkup, Update, UpdateContent,
    },
};

use crate::components::telegram::{subscribe_user, Expecting, HashMapDatabase, HashMapExpecting};

use super::{
    consts::{
        ADD_FILTER, AVAILABLE_RIL100_RAILWAY_MANAGEMENTS, CHANGE_FILTER_BEHAVIOUR,
        CHANGE_FILTER_BEHAVIOUR_ALL, CHANGE_FILTER_BEHAVIOUR_ONE, LOCATION, ONLY_CANCELLATIONS,
        RAILWAY_MANAGEMENT, REMOVE_FILTER, REMOVE_FILTER_PREFIX, SHOW_FILTER,
    },
    model::Filter,
};

#[prepare_listener]
pub async fn callback(context: Context, update: Update) {
    if let Err(e) = match update.content {
        UpdateContent::CallbackQuery(query) => callback_query(context, query).await,
        UpdateContent::Message(message) => callback_message(context, message).await,
        _ => Ok(()),
    } {
        warn!("Error while running callback handler: {e:?}");
    }
}

// got a callback
async fn callback_query(context: Context, query: CallbackQuery) -> Result<(), Box<dyn Error>> {
    let data = query
        .data
        .ok_or("Callback query does not contain any data")?;
    let message = query
        .message
        .ok_or("Callback query does not contain a message")?;

    let database = context
        .data
        .write()
        .get::<HashMapDatabase>()
        .unwrap()
        .clone();
    let connection = database.get_connection().await.unwrap();
    let user_id = subscribe_user(&connection, message.chat.get_id()).await;

    if data == SHOW_FILTER {
        // show all configured filters
        let filters: Vec<Filter> = connection
            .query_one(
                "SELECT filters FROM telegram_user WHERE id = $1",
                &[&user_id],
            )
            .await?
            .get::<_, Vec<serde_json::Value>>(0)
            .into_iter()
            .map(serde_json::from_value::<Filter>)
            .collect::<Result<Vec<Filter>, serde_json::Error>>()?;

        context
            .api
            .send_message(SendMessage::new(
                message.chat.get_id().into(),
                match filters.is_empty() {
                    true => "Du hast bisher keine Filter konfiguriert.".to_string(),
                    false => format!(
                        "Du hast folgende Filter konfiguriert:\n{}",
                        filters
                            .iter()
                            .map(|f| format!("- {f}"))
                            .collect::<Vec<String>>()
                            .join("\n")
                    ),
                },
            ))
            .await?;
    } else if data == ADD_FILTER {
        // add a filter
        let mut loc_btn = InlineKeyboardButton::new("Umkreis um einen Standort", false);
        loc_btn.callback_data = Some(LOCATION.to_string());

        let mut only_cancellations_btn = InlineKeyboardButton::new("Nur Ausfälle anzeigen", false);
        only_cancellations_btn.callback_data = Some(ONLY_CANCELLATIONS.to_string());

        let mut railway_management_btn = InlineKeyboardButton::new("Nach Bahndirektion", false);
        railway_management_btn.callback_data = Some(RAILWAY_MANAGEMENT.to_string());

        let mut markup = InlineKeyboardMarkup::new();
        markup.add_row(vec![loc_btn]);
        markup.add_row(vec![only_cancellations_btn]);
        markup.add_row(vec![railway_management_btn]);

        let mut send_message = SendMessage::new(
            message.chat.get_id().into(),
            "Welche Art Filter möchtest du hinzufügen?",
        );
        send_message.reply_markup = Some(ReplyMarkup::InlineKeyboardMarkup(markup));
        context.api.send_message(send_message).await?;
    } else if data == REMOVE_FILTER {
        // remove a filter
        let filters: Vec<Filter> = connection
            .query_one(
                "SELECT filters FROM telegram_user WHERE id = $1",
                &[&user_id],
            )
            .await?
            .get::<_, Vec<serde_json::Value>>(0)
            .into_iter()
            .map(serde_json::from_value::<Filter>)
            .collect::<Result<Vec<Filter>, serde_json::Error>>()?;

        if filters.is_empty() {
            context
                .api
                .send_message(SendMessage::new(
                    message.chat.get_id().into(),
                    "Du hast bisher keine Filter konfiguriert.",
                ))
                .await?;
        } else {
            let mut markup = InlineKeyboardMarkup::new();
            for filter in filters {
                let mut btn = InlineKeyboardButton::new(filter.get_type(), false);
                btn.callback_data = Some(format!("{REMOVE_FILTER_PREFIX}{}", filter.get_type()));
                markup.add_button(btn);
            }

            let mut send_message = SendMessage::new(
                message.chat.get_id().into(),
                "Welcher Filter soll entfernt werden?",
            );
            send_message.reply_markup = Some(ReplyMarkup::InlineKeyboardMarkup(markup));
            context.api.send_message(send_message).await?;
        }
    } else if data == CHANGE_FILTER_BEHAVIOUR {
        let one_filter_enough: bool = connection
            .query_one(
                "SELECT one_filter_enough FROM telegram_user WHERE id=$1",
                &[&user_id],
            )
            .await?
            .get(0);

        let mut all_btn = InlineKeyboardButton::new("Alle", false);
        all_btn.callback_data = Some(CHANGE_FILTER_BEHAVIOUR_ALL.to_string());

        let mut one_btn = InlineKeyboardButton::new("Einer", false);
        one_btn.callback_data = Some(CHANGE_FILTER_BEHAVIOUR_ONE.to_string());

        let mut markup = InlineKeyboardMarkup::new();
        markup.add_button(all_btn);
        markup.add_button(one_btn);

        let mut send_message = SendMessage::new(message.chat.get_id().into(), format!("Soll ich dir eine Störung nur dann schicken, wenn ALLE oder mindestens EIN Filter erfüllt ist?\nAktuell ist eingestellt: {}", match one_filter_enough {
            true => "Einer",
            false => "Alle"
        }));
        send_message.reply_markup = Some(ReplyMarkup::InlineKeyboardMarkup(markup));
        context.api.send_message(send_message).await?;
    } else if data == CHANGE_FILTER_BEHAVIOUR_ALL || data == CHANGE_FILTER_BEHAVIOUR_ONE {
        connection
            .execute(
                "UPDATE telegram_user SET one_filter_enough=$1 WHERE id=$2",
                &[
                    &match data.as_str() {
                        CHANGE_FILTER_BEHAVIOUR_ALL => false,
                        CHANGE_FILTER_BEHAVIOUR_ONE => true,
                        _ => unreachable!(),
                    },
                    &user_id,
                ],
            )
            .await?;
        context
            .api
            .send_message(SendMessage::new(
                message.chat.get_id().into(),
                "Das Filterverhalten wurde erfolgreich angepasst.",
            ))
            .await?;
    } else if data == LOCATION {
        // add a location based filter
        let expecting_arc = context
            .data
            .write()
            .get::<HashMapExpecting>()
            .unwrap()
            .clone();
        let mut expecting = expecting_arc.lock().await;
        expecting.insert(user_id, Expecting::Location); // we expect the user to send us a location
        context.api.send_message(SendMessage::new(message.chat.get_id().into(), "Schicke mir nun einen Standort. Du kannst danach konfigurieren in welchem Radius um den Standort eine Störung liegen muss, damit ich sie dir schicke.")).await?;
    } else if data == ONLY_CANCELLATIONS {
        let filter = serde_json::to_value(Filter::OnlyCancellations).unwrap();
        connection.execute("UPDATE telegram_user SET filters=array_append(filters, $1) WHERE id=$2 AND NOT filters @> $3", &[
            &filter,
            &user_id,
            &vec![&filter]
        ]).await?;
        context
            .api
            .send_message(SendMessage::new(
                message.chat.get_id().into(),
                "Du erhälst nun nur noch Ausfälle (❌).",
            ))
            .await?;
    } else if data == RAILWAY_MANAGEMENT {
        let expecting_arc = context
            .data
            .write()
            .get::<HashMapExpecting>()
            .unwrap()
            .clone();
        let mut expecting = expecting_arc.lock().await;
        expecting.insert(user_id, Expecting::RailwayManagement); // we expect the user to send us a railway management letter
        context.api.send_message(SendMessage::new(message.chat.get_id().into(), "Schicke mir nun den Buchstaben der Bahndirektion. Das ist der erste Buchstabe des DS100/RIL100 Kürzels.")).await?;
    } else if data.starts_with(REMOVE_FILTER_PREFIX) {
        // remove a location, data will be like this: `{REMOVE_FILTER_PREFIX}{FILTER_TYPE}`
        let filter_type = data.replace(REMOVE_FILTER_PREFIX, "");
        let filters: Vec<Filter> = connection
            .query_one(
                "SELECT filters FROM telegram_user WHERE id = $1",
                &[&user_id],
            )
            .await?
            .get::<_, Vec<serde_json::Value>>(0)
            .into_iter()
            .map(serde_json::from_value::<Filter>)
            .collect::<Result<Vec<Filter>, serde_json::Error>>()?;
        connection
            .execute(
                "UPDATE telegram_user SET filters=$1 WHERE id=$2",
                &[
                    &filters
                        .iter()
                        .filter(|filter| filter.get_type() != filter_type) // filter all filters out that are of filter_type type
                        .map(|f| serde_json::to_value(f).unwrap())
                        .collect::<Vec<serde_json::Value>>(),
                    &user_id,
                ],
            )
            .await?;
        context
            .api
            .send_message(SendMessage::new(
                message.chat.get_id().into(),
                "Der/Die Filter wurden erfolgreich gelöscht.",
            ))
            .await?;
    } else {
        Err(format!("Callback query data '{data}' is not valid data."))?;
    }

    context
        .api
        .answer_callback_query(AnswerCallbackQuery::new(query.id))
        .await?;

    Ok(())
}

// got a message
async fn callback_message(context: Context, message: Message) -> Result<(), Box<dyn Error>> {
    let database = context
        .data
        .write()
        .get::<HashMapDatabase>()
        .unwrap()
        .clone();
    let connection = database.get_connection().await.unwrap();
    let user_id = subscribe_user(&connection, message.chat.get_id()).await;
    let expecting_arc = context
        .data
        .write()
        .get::<HashMapExpecting>()
        .unwrap()
        .clone();
    let mut expectings = expecting_arc.lock().await;

    if let Some(expecting) = expectings.get(&user_id) {
        if matches!(expecting, Expecting::Location) {
            // we expect a location
            if let MessageContent::Location { content } = message.content {
                expectings.insert(
                    user_id,
                    Expecting::LocationRange {
                        lon: content.longitude,
                        lat: content.latitude,
                    },
                );
                context.api.send_message(SendMessage::new(message.chat.get_id().into(), "In welchem Radius (in Kilometern) soll eine Störung um den Standort liegen?")).await?;
            } else {
                // expecting a location, but message is not a location
                return Ok(());
            }
        } else if let Expecting::LocationRange { lon, lat } = expecting {
            // we expect a message with a range
            if let MessageContent::Text {
                content,
                entities: _,
            } = message.content
            {
                context
                    .api
                    .send_message(SendMessage::new(
                    message.chat.get_id().into(),
                        match content.parse::<u16>() {
                            Ok(range) => {
                                connection.execute("UPDATE telegram_user SET filters=array_append(filters, $1) WHERE id=$2", &[&serde_json::to_value(Filter::Location { x: *lon, y: *lat, range }).unwrap(), &user_id]).await?;
                                expectings.remove_entry(&user_id);
                                "Der Filter wurde erfolgreich erstellt."
                            }
                            Err(_) => {
                                "Der Radius sollte schon eine Zahl sein."
                            }
                })).await?;
            } else {
                // expecting text, but message is not text
                return Ok(());
            }
        } else if matches!(expecting, Expecting::RailwayManagement) {
            if let MessageContent::Text {
                content,
                entities: _,
            } = message.content
            {
                let first_char = content.chars().next();
                if content.len() > 1
                    || !first_char
                        .map(|c| {
                            AVAILABLE_RIL100_RAILWAY_MANAGEMENTS.contains(&c.to_ascii_uppercase())
                        })
                        .unwrap_or(false)
                {
                    context.api.send_message(SendMessage::new(
                        message.chat.get_id().into(),
                        format!("Bitte schicke mir den ersten Buchstaben einer Bahndirektion. Zur Verfügung stehen: {}", AVAILABLE_RIL100_RAILWAY_MANAGEMENTS.iter().map(|c| c.to_string()).collect::<Vec<String>>().join(", ")))
                    ).await?;
                    return Ok(());
                }

                connection
                    .execute(
                        "UPDATE telegram_user SET filters=array_append(filters, $1) WHERE id=$2",
                        &[
                            &serde_json::to_value(Filter::RailwayManagement {
                                letter: first_char.unwrap(),
                            })
                            .unwrap(),
                            &user_id,
                        ],
                    )
                    .await?;
                expectings.remove_entry(&user_id);
                context
                    .api
                    .send_message(SendMessage::new(
                        message.chat.get_id().into(),
                        "Der Filter wurde erfolgreich erstellt.",
                    ))
                    .await?;
            } else {
                // expecting text, but message is not text
                return Ok(());
            }
        }
    }

    Ok(())
}
