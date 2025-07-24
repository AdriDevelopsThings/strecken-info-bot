use serde::Deserialize;

#[derive(Deserialize)]
pub struct Infrastructure {
    pub id: i32,
    #[serde(rename = "fahrplanjahr")]
    pub year: i32,
    #[serde(rename = "ordnungsrahmen", default)]
    pub data: Option<InfrastructureData>,
}

#[derive(Deserialize)]
pub struct InfrastructureData {
    #[serde(rename = "betriebsstellen")]
    pub stations: Vec<InfrastructureStation>,
}

#[derive(Deserialize)]
pub struct InfrastructureStation {
    pub ds100: String,
    #[serde(rename = "geo_koordinaten", default)]
    pub coordinates: Option<InfrastructureStationCoordinates>,
}

#[derive(Deserialize)]
pub struct InfrastructureStationCoordinates {
    #[serde(rename = "breite")]
    pub lat: f64,
    #[serde(rename = "laenge")]
    pub lon: f64,
}
