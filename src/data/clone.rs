use crate::data::DataDisruption;

pub trait DataDisruptionClone {
    fn clone_box(&self) -> Box<dyn DataDisruption>;
}

impl<T> DataDisruptionClone for T
where
    T: 'static + DataDisruption + Clone,
{
    fn clone_box(&self) -> Box<dyn DataDisruption> {
        Box::new(self.clone())
    }
}

impl Clone for Box<dyn DataDisruption> {
    fn clone(&self) -> Self {
        self.clone_box()
    }
}
