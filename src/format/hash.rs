use strecken_info::disruptions::Disruption;

pub fn format_hash(disruption: &Disruption) -> String {
    let mut input = String::new();
    input += "p:";
    input += "l:";
    input += disruption
        .stations
        .iter()
        .map(|location| location.ril100.clone())
        .collect::<Vec<String>>()
        .join(";")
        .as_str();
    input += "r:";
    input += disruption.regions.to_vec().join(";").as_str();
    input += "i:";
    input += &disruption.cause;
    input += &disruption.subcause.clone().unwrap_or_default();
    input += &disruption
        .effects
        .iter()
        .map(|effect| format!("{}", effect.effect))
        .collect::<Vec<String>>()
        .join(",");
    input += "t:";
    input += &disruption.text;

    format!("{:x}", md5::compute(input.as_bytes()))
}
