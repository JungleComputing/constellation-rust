use std::fmt::{Debug, Display};

pub trait PayloadTrait: Sync + Send + Debug + PayloadTraitClone + Display + mopa::Any {}

pub trait PayloadTraitClone {
    fn clone_box(&self) -> Box<dyn PayloadTrait>;
}

impl Clone for Box<dyn PayloadTrait> {
    fn clone(&self) -> Box<dyn PayloadTrait> {
        self.clone_box()
    }
}

mopafy!(PayloadTrait);
