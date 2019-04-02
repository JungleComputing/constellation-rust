use std::fmt::{Debug, Display};

pub trait MessageTrait: Sync + Send + Debug + MessageTraitClone + Display {}

pub trait MessageTraitClone {
    fn clone_box(&self) -> Box<dyn MessageTrait>;
}

impl Clone for Box<dyn MessageTrait> {
    fn clone(&self) -> Box<dyn MessageTrait> {
        self.clone_box()
    }
}
