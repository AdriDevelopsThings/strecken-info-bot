use strecken_info::Disruption;

use crate::user::User;

pub enum DisruptionFilter {
    TooLongDisruption { days: u8 },
    NotPlanned,
}

impl DisruptionFilter {
    pub fn filter(&self, disruption: &Disruption) -> bool {
        match self {
            Self::TooLongDisruption { days } => {
                (disruption.end_date - disruption.start_date).num_days() < *days as i64
            }
            Self::NotPlanned => !disruption.planned,
        }
    }

    pub fn filters(filters: &[DisruptionFilter], disruption: &Disruption) -> bool {
        filters
            .iter()
            .map(|filter| filter.filter(disruption))
            .all(|x| x)
    }
}

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
