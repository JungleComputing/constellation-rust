//! Single threaded implementation of Constellation.

use super::constellation_identifier::ConstellationIdentifier;
use super::work_queue::WorkQueue;
use super::activity;
use super::super::constellation;
use super::super::constellation::ConstellationTrait;
use super::super::implementation::error::ConstellationError;
use super::super::activity::ActivityTrait;
use super::super::event::Event;
use super::super::constellation_config::ConstellationConfiguration;

use mpi::topology::SystemCommunicator;
use mpi::topology::Communicator;
use mpi::environment::Universe;
use super::super::context::Context;

/// A single threaded Constellation instance containing.
///
/// * `identifier` - ConstellationIdentifier used to identify this
/// constellation instance.
/// * `fresh` - Work_queue containing fresh work that anyone can steal.
/// * `stolen` - Work_queue containing stolen work from other Constellation
/// instances.
pub struct SingleThreadConstellation<'a> {
    identifier: ConstellationIdentifier,
    fresh: WorkQueue<Box<dyn activity::ActivityTrait>>,
    stolen: WorkQueue<Box<dyn activity::ActivityTrait>>,
    active: bool,
    universe: Universe,
    parent: Option<&'a constellation::ConstellationTrait>
}

impl<'a> constellation::ConstellationTrait for SingleThreadConstellation<'a> {
    fn activate(&mut self) -> Result<bool, ConstellationError> {
        // TODO Activate Constellation instance
        self.active = true;

        return Result::Ok(true);
    }

    fn submit(&self, activity: &ActivityTrait, context: Context, can_be_stolen: bool, expects_events: bool) {
        unimplemented!()
    }

    fn send(&self, e: Event) {
        unimplemented!()
    }

    fn done(&self) -> Result<bool, ConstellationError> {
        unimplemented!()
    }

    fn identifier(&self) -> ConstellationIdentifier {
        self.identifier.clone()
    }

    fn is_master(&self) -> Result<bool, ConstellationError> {
        if self.rank() == 0 { Ok(true) } else { Ok(false) }
    }

    fn nodes(&self) -> i32 {
        self.world().size()
    }

    fn generate_identifier(&self) -> ConstellationIdentifier{
        // Check if there is a multithreaded Constellation running
        if self.parent.is_none() {
            // Has no parent, this is the top-level instance of Constellation

            // Consider figuring out better way to do this than to clone()
            // the entire universe structure
            return ConstellationIdentifier::new(&self.universe);
        }

        // Call parent method ConstellationIdentifier
        self.parent.unwrap().generate_identifier()
    }
}

impl<'a> SingleThreadConstellation<'a> {

    /// Create a new single threaded constellation instance
    ///
    /// # Returns
    /// * `SingleThreadedConstellation` - New single threaded Constellation
    /// instance
    pub fn new(config: Box<ConstellationConfiguration>) -> SingleThreadConstellation<'a> {
        SingleThreadConstellation {
            identifier: ConstellationIdentifier::new_empty(),
            fresh: WorkQueue::new(),
            stolen: WorkQueue::new(),
            active: false,
            universe: mpi::initialize().unwrap(),
            parent: None
        }
    }

    pub fn new_with_parent(config: Box<ConstellationConfiguration>,
                           parent: &'a constellation::ConstellationTrait)
        -> SingleThreadConstellation<'a> {

        let new_const = SingleThreadConstellation {
            identifier: ConstellationIdentifier::new_empty(),
            fresh: WorkQueue::new(),
            stolen: WorkQueue::new(),
            active: false,
            universe: mpi::initialize().unwrap(),
            parent: Some(parent),
        };

        new_const.generate_identifier();
        new_const

    }

    fn rank(&self) -> i32{
        self.world().rank()
    }

    fn world(&self) -> SystemCommunicator {
        self.universe.world()
    }
}