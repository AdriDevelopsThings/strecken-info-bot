use html_escape::encode_text;
use strecken_info::Disruption;

pub fn disruption_to_string(disruption: &Disruption) -> String {
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
    let head = impacts.join(", ") + "\n" + disruption.head.as_str();
    let mut events = disruption
        .events
        .iter()
        .map(|event| {
            format!(
                "{} bis vsl. {}",
                event.start_time.format("%d.%m.%Y %H:%M"),
                event.end_time.format("%d.%m.%Y %H:%M")
            )
        })
        .collect::<Vec<String>>();
    events.dedup();
    let times = events.join("\n");

    let text = disruption
        .text
        .clone()
        .unwrap_or_default()
        .replace("<br/>", "\n")
        .replace("<br>", "\n")
        .replace("<br />", "\n");

    format!(
        "<i><u>Ort: {location}</u></i>\n<b>{head}</b>\n\n{times}\n\n{}\nPriorität: {}",
        encode_text(&text),
        disruption.prio
    )
}
