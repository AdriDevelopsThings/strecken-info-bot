use std::fmt::Display;

use serde::{Deserialize, Serialize};

use crate::{
    components::telegram::filter::strecken_info::StreckenInfoFilter,
    data::{strecken_info::StreckenInfoDisruption, DataDisruption},
    TrassenfinderApi,
};

#[derive(Serialize, Deserialize)]
#[serde(tag = "data_source", rename_all = "snake_case")]
pub enum Filter {
    StreckenInfo(StreckenInfoFilter),
}

impl Filter {
    pub async fn filter(
        &self,
        disruption: &dyn DataDisruption,
        trassenfinder: &Option<TrassenfinderApi>,
    ) -> bool {
        if let Some(disruption) = disruption.as_any().downcast_ref::<StreckenInfoDisruption>() {
            let Self::StreckenInfo(filter) = self;
            return filter.filter(&disruption.disruption, trassenfinder).await;
        } else {
            unreachable!("Disruption type has not implemented in the filter yet.");
        }

        // should return true otherwise
    }

    pub fn get_id(&self) -> String {
        match self {
            Self::StreckenInfo(filter) => format!("stinf_{}", filter.get_id()),
        }
    }
}

impl Display for Filter {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::StreckenInfo(filter) => write!(f, "Strecken.Info: {filter}"),
        }
    }
}
