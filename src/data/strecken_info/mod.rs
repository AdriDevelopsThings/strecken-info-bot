use std::any::Any;

pub use fetcher::start_fetching;
use strecken_info::disruptions::Disruption;

use crate::{
    components::ComponentType,
    data::{strecken_info::change::DisruptionPart, DataDisruption, DatabaseRepresentation},
};

mod change;
mod fetcher;
mod format;

pub const STRECKEN_INFO_TYPE: &str = "strecken_info";

#[derive(Clone)]
pub struct StreckenInfoDisruption {
    pub disruption: Disruption,
    pub changes: Vec<DisruptionPart>,
}

impl DataDisruption for StreckenInfoDisruption {
    fn get_type(&self) -> &'static str {
        STRECKEN_INFO_TYPE
    }

    fn format(&self, component_type: ComponentType, updated: bool) -> String {
        match component_type {
            ComponentType::Telegram => format::format_to_html(&self.disruption, updated),
            ComponentType::Mastodon => {
                format::format_to_text(&self.disruption, &self.changes, updated)
            }
        }
    }

    fn format_tws(&self) -> String {
        format!(
            "{} {} {}",
            self.disruption.cause,
            self.disruption.subcause.clone().unwrap_or_default(),
            self.disruption.text
        )
    }

    fn get_database_repr(&self) -> DatabaseRepresentation {
        DatabaseRepresentation {
            key: self.disruption.key.clone(),
            start_time: self.disruption.period.start,
            end_time: self.disruption.period.end,
            json: serde_json::to_value(&self.disruption).unwrap(),
        }
    }

    fn as_any(&self) -> &dyn Any {
        self
    }
}
