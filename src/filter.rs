use strecken_info::Disruption;

pub enum Filter {
    PrioFilter { min: u8 },
    PlannedFilter,
}

impl Filter {
    pub fn filter(&self, disruption: &Disruption) -> bool {
        match self {
            Self::PrioFilter { min } => &disruption.prio <= min,
            Self::PlannedFilter => !disruption.planned,
        }
    }

    pub fn filters(filters: &[Filter], disruption: &Disruption) -> bool {
        filters
            .iter()
            .map(|filter| filter.filter(disruption))
            .all(|x| x)
    }
}
