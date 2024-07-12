use strecken_info::disruptions::Disruption;

use crate::format::hash::format_hash;

pub const ALL_DISRUPTION_PARTS: &[DisruptionPart] = &[
    DisruptionPart::Cause,
    DisruptionPart::Effects,
    DisruptionPart::Locations,
    DisruptionPart::Times,
    DisruptionPart::Text,
];

#[derive(Clone, PartialEq)]
pub enum DisruptionPart {
    Cause,
    Effects,
    Locations,
    Times,
    Text,
}

pub fn get_disruption_changes(
    database_disruption: Option<Disruption>,
    hash: String,
    new_disruption: &Disruption,
) -> Vec<DisruptionPart> {
    if let Some(database_disruption) = database_disruption {
        let mut parts = Vec::new();

        if database_disruption.cause != new_disruption.cause
            || database_disruption.subcause != new_disruption.subcause
        {
            parts.push(DisruptionPart::Cause);
        }

        if database_disruption.effects != new_disruption.effects {
            parts.push(DisruptionPart::Effects);
        }

        if database_disruption.stations != new_disruption.stations
            || database_disruption.regions != new_disruption.regions
        {
            parts.push(DisruptionPart::Locations);
        }

        if database_disruption.period != new_disruption.period {
            parts.push(DisruptionPart::Times);
        }

        if database_disruption.text != new_disruption.text {
            parts.push(DisruptionPart::Text);
        }

        return parts;
    }

    let new_hash = format_hash(new_disruption);
    if new_hash == hash {
        Vec::new()
    } else {
        ALL_DISRUPTION_PARTS.to_vec()
    }
}
