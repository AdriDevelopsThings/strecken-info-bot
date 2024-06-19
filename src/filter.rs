use strecken_info::disruptions::Disruption;

pub enum DisruptionFilter {
    TooLongDisruption { days: u8 },
}

impl DisruptionFilter {
    pub fn filter(&self, disruption: &Disruption) -> bool {
        match self {
            Self::TooLongDisruption { days } => {
                (disruption.period.end - disruption.period.start).num_days() < *days as i64
            }
        }
    }

    pub fn filters(filters: &[DisruptionFilter], disruption: &Disruption) -> bool {
        filters
            .iter()
            .map(|filter| filter.filter(disruption))
            .all(|x| x)
    }
}
