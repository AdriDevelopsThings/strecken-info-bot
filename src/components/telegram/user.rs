use bb8_postgres::tokio_postgres::Row;
use strecken_info::disruptions::{Disruption, TrackRestriction};

use super::{epsg_3857_to_epsg_4326, epsg_4326_distance_km, Filter};

pub struct User {
    pub chat_id: i64,
    pub trigger_warnings: Vec<String>,
    pub filters: Vec<Filter>,
    pub one_filter_enough: bool,
}

impl User {
    pub fn from_row(value: &Row) -> Result<Self, serde_json::Error> {
        Ok(Self {
            chat_id: value.get(1),
            trigger_warnings: value.get::<_, Vec<String>>(2),
            filters: value
                .get::<_, Vec<serde_json::Value>>(4)
                .into_iter()
                .map(serde_json::from_value::<Filter>)
                .collect::<Result<Vec<Filter>, serde_json::Error>>()?,
            one_filter_enough: value.get(5),
        })
    }

    pub fn is_trigger(&self, message: &str) -> Option<String> {
        let message = message.to_lowercase();
        for trigger in &self.trigger_warnings {
            if message.contains(&trigger.to_lowercase()) {
                return Some(trigger.to_owned());
            }
        }
        None
    }

    pub fn is_filtered(&self, disruption: &Disruption) -> bool {
        if self.filters.is_empty() {
            return false;
        }

        let mut filters_mapped = self.filters.iter().map(|filter| match filter {
            Filter::Location { x, y, range } => {
                return disruption
                    .coordinates
                    .iter()
                    .map(|coordinate| {
                        if !coordinate.x.is_normal() || !coordinate.y.is_normal() {
                            return false;
                        }

                        let (coordinate_x, coordinate_y) =
                            epsg_3857_to_epsg_4326(coordinate.x, coordinate.y);
                        let distance = epsg_4326_distance_km(coordinate_x, coordinate_y, *x, *y);
                        distance <= (*range as f64)
                    })
                    .any(|x| x);
            }
            Filter::OnlyCancellations => disruption.track_restriction == TrackRestriction::Severe,
            Filter::RailwayManagement { letter } => disruption
                .stations
                .iter()
                .any(|station| station.ril100.starts_with(*letter)),
        });

        match self.one_filter_enough {
            true => !filters_mapped.any(|x| x),
            false => !filters_mapped.all(|x| x),
        }
    }
}
