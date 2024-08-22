use std::fmt::Display;

use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum Filter {
    // coordinates in epsg 3857
    // range in kilometers
    Location { x: f64, y: f64, range: u16 },
    OnlyCancellations,
    // "Bahndirektion" (The first letter of ril100)
    RailwayManagement { letter: char },
}

impl Filter {
    pub fn get_type(&self) -> &'static str {
        match self {
            Self::Location {
                x: _,
                y: _,
                range: _,
            } => "Standort",
            Self::OnlyCancellations => "Nur Ausfälle",
            Self::RailwayManagement { letter: _ } => "Bahndirektion",
        }
    }
}

impl Display for Filter {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Location { x, y, range } => {
                write!(
                    f,
                    "{}: x = {x}, y = {y}, Abstand = {range}km",
                    self.get_type()
                )
            }
            Self::OnlyCancellations => write!(f, "{}", self.get_type()),
            Self::RailwayManagement { letter } => {
                write!(f, "{}: RIL100 beginnt mit {letter}", self.get_type())
            }
        }
    }
}
