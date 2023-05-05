use strecken_info::Disruption;

pub enum Filter {
    Planned,
    TooLongDisruption { days: u8 },
}

impl Filter {
    pub fn filter(&self, disruption: &Disruption) -> bool {
        match self {
            Self::Planned => !disruption.planned,
            Self::TooLongDisruption { days } => {
                (disruption.end_date - disruption.start_date).num_days() < *days as i64
            }
        }
    }

    pub fn filters(filters: &[Filter], disruption: &Disruption) -> bool {
        filters
            .iter()
            .map(|filter| filter.filter(disruption))
            .all(|x| x)
    }
}
