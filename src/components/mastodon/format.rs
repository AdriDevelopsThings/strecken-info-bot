use strecken_info::disruptions::Disruption;

use crate::format::{
    format_text,
    partial_format::{get_cause, get_location, get_product_effects, get_times},
};

pub(super) fn format(disruption: &Disruption, changed: bool) -> String {
    let location = get_location(disruption, Some(8));
    let cause = get_cause(disruption);
    let effects = get_product_effects(disruption);
    let prefix = if disruption.expired {
        "✅ Beendet: "
    } else {
        match changed {
            true => "Update: ",
            false => "⚠️",
        }
    };
    let times = get_times(disruption);
    format!(
        "{prefix}{location}\n{cause}\n{effects}\n{}\n{times}\n{}",
        format_text(&disruption.cause),
        format_text(&disruption.text.clone())
    )
}
