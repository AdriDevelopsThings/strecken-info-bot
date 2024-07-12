use strecken_info::disruptions::{Disruption, TrackRestriction};

use crate::{
    change::DisruptionPart,
    format::{
        format_text,
        partial_format::{get_cause, get_location, get_product_effects, get_times, is_expired},
    },
};

pub(super) fn format(disruption: &Disruption, changes: &[DisruptionPart], update: bool) -> String {
    let prefix = if is_expired(disruption) {
        "✅ Beendet: "
    } else {
        match update {
            true => "Update: ",
            false => match disruption.track_restriction {
                TrackRestriction::Severe => "❌ ",
                TrackRestriction::Slight => "⚠️ ",
            },
        }
    };

    let mut str = prefix.to_string();
    if changes.contains(&DisruptionPart::Locations) {
        str += &format!("{}\n", get_location(disruption, Some(8)));
    }
    if changes.contains(&DisruptionPart::Cause) {
        str += &format!("{}\n", format_text(&get_cause(disruption)));
    }
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
