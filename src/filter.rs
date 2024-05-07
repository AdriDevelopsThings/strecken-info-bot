use strecken_info::Disruption;

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
