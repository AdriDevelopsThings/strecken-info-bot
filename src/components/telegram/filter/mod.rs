use futures::future::join_all;

use crate::{components::telegram::user::User, data::DataDisruption, TrassenfinderApi};

mod model;
mod strecken_info;

pub use model::Filter;
pub use strecken_info::StreckenInfoFilter;

impl User {
    pub async fn is_filtered(
        &self,
        disruption: &dyn DataDisruption,
        trassenfinder: &Option<TrassenfinderApi>,
    ) -> bool {
        if self.filters.is_empty() {
            return false;
        }

        let mut filters_mapped = join_all(
            self.filters
                .iter()
                .map(|filter| filter.filter(disruption, trassenfinder)),
        )
        .await
        .into_iter();

        match self.one_filter_enough {
            true => !filters_mapped.any(|x| x),
            false => !filters_mapped.all(|x| x),
        }
    }
}
