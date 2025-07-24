use bb8_postgres::tokio_postgres::Row;

use crate::components::telegram::filter::Filter;

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
}
