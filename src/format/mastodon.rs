use chrono::Utc;
use strecken_info::Disruption;

use crate::format::{
    format_text,
    partial_format::{get_end, get_events, get_impacts, get_location},
};

pub fn format(disruption: &Disruption, changed: bool) -> String {
    let location = get_location(disruption, Some(8));
    let impacts = get_impacts(disruption).join("\n");
    let end = get_end(disruption);
    let prefix = if Utc::now() > end {
        "Beendet: "
    } else {
        match changed {
            true => "Update: ",
            false => "⚠️",
        }
    };
    let times = get_events(disruption, end).join("\n");
    format!(
        "{prefix}{location}\n{impacts}\n{}\n{times}\n{}",
        format_text(&disruption.head),
        format_text(&disruption.text.clone().unwrap_or_default())
    )
}
