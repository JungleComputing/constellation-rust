//! Module for handling Errors and Results
use std::{error, fmt, result};

#[derive(Debug)]
pub struct ConstellationError;

// Result type which can often have Constellation errors
pub type Result<T> = result::Result<T, ConstellationError>;

impl fmt::Display for ConstellationError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "THIS IS AN ERROR")
    }
}

impl error::Error for ConstellationError {
    // TODO Add methods/functions to identify error

    fn cause(&self) -> Option<&error::Error> {
        // Generic error, underlying cause isn't tracked.
        None
    }
}