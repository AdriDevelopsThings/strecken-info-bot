use bb8_postgres::tokio_postgres::Row;

pub struct User {
    pub chat_id: i64,
    pub trigger_warnings: Vec<String>,
}

impl User {
    pub fn from_row(value: &Row) -> Self {
        Self {
            chat_id: value.get(1),
            trigger_warnings: value.get::<_, Vec<String>>(2),
        }
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
