use html_escape::encode_text;
use strecken_info::disruptions::Disruption;

use crate::format::{
    format_text,
    partial_format::{get_cause, get_location, get_prefix, get_product_effects, get_times},
};

pub(super) fn format(disruption: &Disruption, update: bool) -> String {
    let head = format!(
        "{}\n{}",
        get_cause(disruption),
        get_product_effects(disruption)
    );
    format!(
        "{} <i><u>Ort: {}</u></i>\n<b>{}</b>\n\n{}\n\n{}",
        get_prefix(disruption, update),
        get_location(disruption, None),
        encode_text(&format_text(&head)),
        get_times(disruption),
        encode_text(&format_text(&disruption.text))
    )
}
