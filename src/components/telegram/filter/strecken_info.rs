use std::fmt::Display;

use futures::future::join_all;
use serde::{Deserialize, Serialize};
use strecken_info::disruptions::{Disruption, TrackRestriction};

use crate::{
    components::telegram::{epsg_3857_to_epsg_4326, filter_ui::epsg_4326_distance_km},
    normalize_spaces, TrassenfinderApi,
};

#[derive(Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum StreckenInfoFilter {
    // coordinates in epsg 3857
    // range in kilometers
    Location { x: f64, y: f64, range: u16 },
    OnlyCancellations,
    // "Bahndirektion" (The first letter of ril100)
    RailwayManagement { letter: char },
}

impl StreckenInfoFilter {
    pub fn get_id(&self) -> &'static str {
        match self {
            Self::Location {
                x: _,
                y: _,
                range: _,
            } => "loc",
            Self::OnlyCancellations => "canc",
            Self::RailwayManagement { letter: _ } => "mmnt",
        }
    }

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

impl Display for StreckenInfoFilter {
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

impl StreckenInfoFilter {
    pub async fn filter(
        &self,
        disruption: &Disruption,
        trassenfinder: &Option<TrassenfinderApi>,
    ) -> bool {
        match self {
            Self::Location { x, y, range } => {
                let orig_b = disruption.coordinates.iter().any(|coordinate| {
                    if !coordinate.x.is_normal() || !coordinate.y.is_normal() {
                        return false;
                    }

                    let (coordinate_x, coordinate_y) =
                        epsg_3857_to_epsg_4326(coordinate.x, coordinate.y);
                    let distance = epsg_4326_distance_km(coordinate_x, coordinate_y, *x, *y);
                    distance <= (*range as f64)
                });

                if orig_b {
                    orig_b
                } else {
                    // trassenfinder fallback
                    join_all(
                        disruption
                            .stations
                            .iter()
                            .chain(disruption.sections.iter().flat_map(|s| [&s.from, &s.to]))
                            .map(async |station| {
                                if let Some(trassenfinder) = &trassenfinder {
                                    let stations = trassenfinder.stations.read().await;
                                    if let Some(coords) =
                                        stations.get(&normalize_spaces(&station.ril100))
                                    {
                                        let distance =
                                            epsg_4326_distance_km(coords.0, coords.1, *x, *y);
                                        distance <= (*range as f64)
                                    } else {
                                        false
                                    }
                                } else {
                                    false
                                }
                            }),
                    )
                    .await
                    .into_iter()
                    .any(|x| x)
                }
            }
            Self::OnlyCancellations => disruption.track_restriction == TrackRestriction::Severe,
            Self::RailwayManagement { letter } => disruption
                .stations
                .iter()
                .any(|station| station.ril100.starts_with(*letter)),
        }
    }
}
