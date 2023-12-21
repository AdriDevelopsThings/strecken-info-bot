use chrono::{DateTime, Utc};
use chrono_tz::{Europe::Berlin, Tz};
use strecken_info::{Disruption, Product};

pub(super) fn get_location(disruption: &Disruption, max_locations: Option<usize>) -> String {
    if !disruption.locations.is_empty() {
        let mut locations = disruption
            .locations
            .iter()
            .map(|location| {
                format!(
                    "{}{}",
                    location.from.name.clone(),
                    if let Some(to) = &location.to {
                        format!(" - {}", to.name)
                    } else {
                        String::new()
                    }
                )
            })
            .collect::<Vec<String>>();
        let mut add_after_locations = "";
        if let Some(max_locations) = max_locations {
            if locations.len() > max_locations {
                locations = locations[0..max_locations - 1].to_vec();
                add_after_locations = "...";
            }
        }
        locations.join(", ") + add_after_locations
    } else if !disruption.regions.is_empty() {
        disruption
            .regions
            .iter()
            .map(|region| region.name.clone())
            .collect::<Vec<String>>()
            .join(", ")
    } else {
        "Unbekannt".to_string()
    }
}

pub(super) fn get_impacts(disruption: &Disruption) -> Vec<String> {
    let mut impacts = disruption
        .impact
        .clone()
        .unwrap_or_default()
        .iter()
        .map(|impact| impact.impact.clone())
        .collect::<Vec<String>>();
    impacts.dedup();
    impacts
}

pub(super) fn get_product_impacts(disruption: &Disruption) -> Vec<&str> {
    let mut product_impacts = disruption
        .impact
        .clone()
        .unwrap_or_default()
        .iter()
        .map(|impact| match impact.product {
            Product::LongDistance => "SPFV",
            Product::Local => "SPNV",
            Product::Freight => "SGV",
        })
        .collect::<Vec<&str>>();
    product_impacts.dedup();
    product_impacts
}

pub(super) fn get_end(disruption: &Disruption) -> DateTime<Tz> {
    disruption
        .end_date
        .and_time(disruption.end_time)
        .and_local_timezone(Berlin)
        .unwrap()
}

pub(super) fn get_events(disruption: &Disruption, end: DateTime<Tz>) -> Vec<String> {
    let mut events = disruption
        .events
        .iter()
        .map(|event| {
            format!(
                "{} bis {}{}",
                event.start_time.format("%d.%m.%Y %H:%M"),
                match end > Utc::now() {
                    true => "vsl. ",
                    false => "",
                },
                event.end_time.format("%d.%m.%Y %H:%M")
            )
        })
        .collect::<Vec<String>>();
    events.dedup();
    events
}
