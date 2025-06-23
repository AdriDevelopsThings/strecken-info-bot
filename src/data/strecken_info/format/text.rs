use strecken_info::disruptions::Disruption;

use crate::data::strecken_info::{
    change::DisruptionPart,
    format::{
        format_text,
        partial_format::{get_cause, get_location, get_prefix, get_product_effects, get_times},
    },
};

pub fn format(disruption: &Disruption, changes: &[DisruptionPart], update: bool) -> String {
    let mut str = format!(
        "{} {}\n{}\n",
        get_prefix(disruption, update),
        get_location(disruption, Some(8)),
        format_text(&get_cause(disruption))
    );
    if changes.contains(&DisruptionPart::Effects) {
        str += &format!("{}\n", get_product_effects(disruption));
    }
    if changes.contains(&DisruptionPart::Times) {
        str += &format!("{}\n", get_times(disruption));
    }
    if changes.contains(&DisruptionPart::Text) {
        str += &format!("{}\n", format_text(&disruption.text.clone()));
    }

    str
}
