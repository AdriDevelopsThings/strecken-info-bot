use chrono::Utc;
use html_escape::encode_text;
use strecken_info::Disruption;

use crate::format::{
    format_text,
    partial_format::{get_end, get_events, get_impacts, get_location, get_product_impacts},
};

pub fn format(disruption: &Disruption, changed: bool) -> String {
    let location = get_location(disruption, None);

    let impacts = get_impacts(disruption);

    let product_impacts = get_product_impacts(disruption);

    let planned = match disruption.planned {
        true => " (Geplant)",
        false => "",
    };

    let mut head = impacts.join(", ");
    if !product_impacts.is_empty() {
        head += " (";
        head += product_impacts.join(", ").as_str();
        head += ")";
    }
    if !head.is_empty() {
        head += "\n";
    }
    head += &disruption.head;
    head += planned;

    let end = get_end(disruption);

    let events = get_events(disruption, end);
    let times = events.join("\n");

    let text = disruption.text.clone().unwrap_or_default();

    let prefix = match changed {
        true => {
            if Utc::now() > end {
                "Beendet: "
            } else {
                "Update: "
            }
        }
        false => "",
    };
    format!(
        "{prefix}<i><u>Ort: {location}</u></i>\n<b>{}</b>\n\n{times}\n\n{}",
        encode_text(&format_text(&head)),
        encode_text(&format_text(&text))
    )
}
