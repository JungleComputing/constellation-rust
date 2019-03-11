//! Single threaded implementation of Constellation.

use super::constellation_identifier::ConstellationIdentifier;
use super::work_queue::WorkQueue;
use super::activity::Activity;

/// A single threaded Constellation instance containing.
///
/// * `identifier` - ConstellationIdentifier used to identify this
/// constellation instance.
/// * `fresh` - Work_queue containing fresh work that anyone can steal.
/// * `stolen` - Work_queue containing stolen work from other Constellation
/// instances.
pub struct SingleThreadConstellation {
    identifier: ConstellationIdentifier,
    fresh: WorkQueue<Box<Activity>>,
    stolen: WorkQueue<Box<Activity>>,
    active: bool,
}

//
//impl Constellation for SingleThreadConstellation {
//
//}

impl SingleThreadConstellation {

    /// Create a new single threaded constellation instance
    ///
    /// # Returns
    /// * `SingleThreadedConstellation` - New single threaded Constellation
    /// instance
    ///
    /// //TODO Add configuration and properties as function parameters
    pub fn new() -> SingleThreadConstellation {
        SingleThreadConstellation {
            identifier: ConstellationIdentifier::new(),
            fresh: WorkQueue::new(),
            stolen: WorkQueue::new(),
            active: false
        }
    }
}