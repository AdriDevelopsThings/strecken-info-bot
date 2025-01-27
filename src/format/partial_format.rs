use chrono::Utc;
use chrono_tz::Europe::Berlin;
use strecken_info::disruptions::{Disruption, Product, TrackRestriction};

fn format_station(station: &str) -> String {
    station
        .trim()
        .split(' ')
        .filter(|s| !s.is_empty())
        .collect::<Vec<&str>>()
        .join(" ")
}

pub fn get_location(disruption: &Disruption, max_locations: Option<usize>) -> String {
    let mut locations = disruption
        .stations
        .iter()
        .map(|station| format!("{} ({})", format_station(&station.name), station.ril100))
        .collect::<Vec<String>>();

    locations.extend(disruption.sections.iter().map(|section| {
        format!(
            "{} ({}) - {} ({})",
            format_station(&section.from.name),
            section.from.ril100,
            format_station(&section.to.name),
            section.to.ril100
        )
    }));

    locations.extend(disruption.regions.iter().cloned());
    locations.dedup();
    if locations.is_empty() {
        locations.push("Unbekannt".to_string())
    }

    if let Some(max_locations) = max_locations {
        if locations.len() > max_locations {
            locations = locations[0..max_locations - 1].to_vec();
            locations.push("...".to_string());
        }
    }

    locations.join(", ")
}

pub fn get_cause(disruption: &Disruption) -> String {
    format!(
        "{}{}",
        disruption.cause,
        match &disruption.subcause {
            Some(subcause) => match subcause.is_empty() {
                true => String::new(), // subcause = ""
                false => format!(" - {subcause}"),
            },
            None => String::new(),
        }
    )
}

pub fn get_product_effects(disruption: &Disruption) -> String {
    let mut product_effects = disruption
        .effects
        .iter()
        .map(|effect| {
            format!(
                "{} ({})",
                effect.effect,
                effect
                    .product
                    .iter()
                    .map(|product| match product {
                        Product::LongDistance => "SPFV",
                        Product::Local => "SPNV",
                        Product::Freight => "SGV",
                    })
                    .collect::<Vec<&str>>()
                    .join(", ")
            )
        })
        .collect::<Vec<String>>();
    product_effects.dedup();
    product_effects.join(", ")
}

pub fn get_times(disruption: &Disruption) -> String {
    format!(
        "{} bis {}",
        disruption.period.start.format("%d.%m.%Y %H:%M"),
        disruption.period.end.format("%d.%m.%Y %H:%M")
    )
}

pub fn is_expired(disruption: &Disruption) -> bool {
    disruption.expired || Utc::now() > disruption.period.end.and_local_timezone(Berlin).unwrap()
}

pub fn get_prefix(disruption: &Disruption, update: bool) -> &'static str {
    if is_expired(disruption) {
        "✅ Beendet:"
    } else {
        match update {
            true => "Update:",
            false => match disruption.track_restriction {
                TrackRestriction::Severe => "❌",
                TrackRestriction::Slight => "⚠️",
            },
        }
    }
}
