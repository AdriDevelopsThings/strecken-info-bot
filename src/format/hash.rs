use strecken_info::Disruption;

pub fn format(disruption: &Disruption) -> String {
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
