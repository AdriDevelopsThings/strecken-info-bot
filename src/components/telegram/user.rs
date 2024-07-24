use bb8_postgres::tokio_postgres::Row;
use strecken_info::disruptions::Disruption;

use super::Filter;

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

        let mut filters_mapped = self.filters.iter().map(|filter| {
            match filter {
                Filter::Location { x, y, range } => {
                    for coordinate in &disruption.coordinates {
                        if !coordinate.x.is_normal() || !coordinate.y.is_normal() {
                            continue;
                        }

                        // distance between (x, y) and coordinate <= range
                        return f64::sqrt(
                            f64::powi(x - coordinate.x, 2) + f64::powi(y - coordinate.y, 2),
                        ) <= (*range as f64 * 1000f64); // range from km to m
                    }
                }
            }
            true
        });

        match self.one_filter_enough {
            true => !filters_mapped.any(|x| x),
            false => !filters_mapped.all(|x| x),
        }
    }
}
