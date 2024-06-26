use chrono::Utc;
use chrono_tz::Europe::Berlin;
use strecken_info::disruptions::{Disruption, Product};

pub fn get_location(disruption: &Disruption, max_locations: Option<usize>) -> String {
    if !disruption.stations.is_empty() {
        let mut locations = disruption
            .stations
            .iter()
            .map(|station| {
                format!(
                    "{} ({})",
                    station
                        .name
                        .trim()
                        .split(' ')
                        .filter(|s| !s.is_empty())
                        .collect::<Vec<&str>>()
                        .join(" "),
                    station.ril100
                )
            })
            .collect::<Vec<String>>();
        locations.dedup();
        let mut add_after_locations = "";
        if let Some(max_locations) = max_locations {
            if locations.len() > max_locations {
                locations = locations[0..max_locations - 1].to_vec();
                add_after_locations = "...";
            }
        }
        locations.join(", ") + add_after_locations
    } else if !disruption.regions.is_empty() {
        disruption.regions.to_vec().join(", ")
    } else {
        "Unbekannt".to_string()
    }
}

pub fn get_cause(disruption: &Disruption) -> String {
    format!(
        "{}{}",
        disruption.cause,
        match &disruption.subcause {
            Some(subcause) => format!(" - {subcause}"),
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
