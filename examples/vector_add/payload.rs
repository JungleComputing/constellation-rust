//! Payload struct for passing data between activities

use std::fmt;

use constellation_rust::payload::{PayloadTrait, PayloadTraitClone};

#[derive(Debug, Clone)]
pub struct Payload {
    pub vec: Vec<i32>,
}

impl PayloadTrait for Payload {}

impl PayloadTraitClone for Payload {
    fn clone_box(&self) -> Box<dyn PayloadTrait> {
        Box::new(self.clone())
    }
}

/// Only print the first 30 elements of the array to not overflow screen
impl fmt::Display for Payload {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let length = if self.vec.len() < 30 {
            self.vec.len()
        } else {
            30
        };
        write!(f, "vec1: {:?}", &self.vec[0..length as usize])
    }
}
