use strecken_info::Disruption;

use super::user::User;

pub enum UserFilter {
    Planned,
}

impl UserFilter {
    pub fn filter(&self, disruption: &Disruption, user: &User) -> bool {
        match self {
            Self::Planned => !disruption.planned || user.show_planned_disruptions,
        }
    }

    pub fn filters(filters: &[UserFilter], disruption: &Disruption, user: &User) -> bool {
        filters
            .iter()
            .map(|filter| filter.filter(disruption, user))
            .all(|x| x)
    }
}
