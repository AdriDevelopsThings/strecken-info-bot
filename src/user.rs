use r2d2_sqlite::rusqlite::Row;

pub struct User {
    pub id: i32,
    pub chat_id: i64,
    pub trigger_warnings: Vec<String>,
    pub show_planned_disruptions: bool,
}

impl User {
    pub fn from_row(value: &Row) -> Result<Self, r2d2_sqlite::rusqlite::Error> {
        Ok(Self {
            id: value.get(0)?,
            chat_id: value.get(1)?,
            trigger_warnings: value
                .get::<usize, String>(2)?
                .split(',')
                .map(|s| s.to_owned())
                .filter(|s| !s.is_empty())
                .collect(),
            show_planned_disruptions: value.get(3)?,
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
