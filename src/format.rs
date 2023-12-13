use chrono::Utc;
use chrono_tz::Europe::Berlin;
use html_escape::encode_text;
use regex::Regex;
use strecken_info::{Disruption, Product};

pub fn hash_disruption(disruption: &Disruption) -> String {
    let mut input = String::new();
    input += "p:";
    input += match disruption.planned {
        true => "1",
        false => "0",
    };
    input += "l:";
    input += disruption
        .locations
        .iter()
        .map(|location| {
            location.from.name.clone()
                + location
                    .to
                    .clone()
                    .map(|l| l.name)
                    .unwrap_or_default()
                    .as_str()
        })
        .collect::<Vec<String>>()
        .join(";")
        .as_str();
    input += "r:";
    input += disruption
        .regions
        .iter()
        .map(|region| region.name.clone())
        .collect::<Vec<String>>()
        .join(";")
        .as_str();
    input += "i:";
    input += disruption
        .impact
        .clone()
        .map(|impacts| {
            impacts
                .into_iter()
                .map(|impact| impact.impact)
                .collect::<Vec<String>>()
                .join(";")
        })
        .unwrap_or_default()
        .as_str();
    input += "e:";
    input += disruption
        .events
        .iter()
        .map(|event| format!("{}-{}", event.start_time, event.end_time))
        .collect::<Vec<String>>()
        .join(";")
        .as_str();
    input += "h:";
    input += &disruption.head;
    input += "t:";
    input += disruption.text.clone().unwrap_or_default().as_str();

    format!("{:x}", md5::compute(input.as_bytes()))
}

fn format_text(text: &str) -> String {
    let text_regex = Regex::new("<br */?>").unwrap();
    text_regex.replace_all(text, "\n").to_string()
}

pub fn disruption_to_string(disruption: &Disruption, changed: bool) -> String {
    let location = if !disruption.locations.is_empty() {
        disruption
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
            .collect::<Vec<String>>()
            .join(", ")
    } else if !disruption.regions.is_empty() {
        disruption
            .regions
            .iter()
            .map(|region| region.name.clone())
            .collect::<Vec<String>>()
            .join(", ")
    } else {
        "Unbekannt".to_string()
    };

    let mut impacts = disruption
        .impact
        .clone()
        .unwrap_or_default()
        .iter()
        .map(|impact| impact.impact.clone())
        .collect::<Vec<String>>();
    impacts.dedup();

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

    let end = disruption
        .end_date
        .and_time(disruption.end_time)
        .and_local_timezone(Berlin)
        .unwrap();

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
