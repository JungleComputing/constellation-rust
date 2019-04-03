extern crate crossbeam;
extern crate mpi;

use std::sync::Arc;
use std::sync::Mutex;

use super::super::communication::mpi_info;
use crate::activity::ActivityTrait;
use crate::activity_identifier::ActivityIdentifier;
use crate::constellation::ConstellationTrait;
use crate::constellation_identifier::ConstellationIdentifier;
use crate::context::Context;
use crate::event::Event;
use crate::implementation::activity_wrapper::ActivityWrapper;
use crate::implementation::activity_wrapper::ActivityWrapperTrait;
use crate::implementation::error::ConstellationError;
use crate::constellation_config::ConstellationConfiguration;

use crossbeam::deque;
use mpi::environment::Universe;


/// This data structure is used in order to share a constellation instance
/// between both the Executor and SingleThreadedConstellation (initiated by
/// application developer).
///
/// All the main Constellation logic is performed here, such as sending
/// events and submitting activities.
///
/// # Members
/// * `identifier` - Identifier for this constellation instance, must be
/// protected with mutex since it contains dynamic methods for ID generation
/// * `universe` - MPI struct containing information about all nodes,
/// threads and connections in the running Constellation instance.
/// * `work_queue` - Queue used to share activities with the executor thread
/// * `event_queue` - Queue used to share events with the executor thread
/// * `parent` - Possible parent constellation instance, used in multithreading
pub struct InnerConstellation {
    identifier: Arc<Mutex<ConstellationIdentifier>>,
    universe: Universe,
    debug: bool,
    nodes: i32,
    pub work_queue: Arc<Mutex<deque::Injector<Box<dyn ActivityWrapperTrait>>>>,
    pub event_queue: Arc<Mutex<deque::Injector<Box<Event>>>>,
    pub parent: Option<Arc<Mutex<dyn ConstellationTrait>>>,
}

impl ConstellationTrait for InnerConstellation {
    fn activate(&mut self) -> Result<bool, ConstellationError> {
        panic!("This function should never be called from inside inner class");
    }

    fn submit(
        &mut self,
        activity: Arc<Mutex<ActivityTrait>>,
        context: &Context,
        may_be_stolen: bool,
        expects_events: bool,
    ) -> ActivityIdentifier {
        let activity_wrapper = ActivityWrapper::new(
            self.identifier.clone(),
            activity,
            context,
            may_be_stolen,
            expects_events,
        );
        let activity_id = activity_wrapper.activity_identifier().clone();

        if self.debug {
            info!("Submitting activity with ID: {}", &activity_id);
        }

        // Insert ActivityWrapper in injector_queue
        self.work_queue
            .lock()
            .expect("Could not get lock on injector_queue, failed to push activity")
            .push(activity_wrapper);

        activity_id
    }

    /// Perform a send operation with the event specified as argument
    ///
    /// # Arguments
    /// * `e` - Event to send, contains src and destination IDs
    fn send(&mut self, e: Box<Event>) {
        if self.debug {
            info!("Send Event: {} -> {}", e.get_src(), e.get_dst());
        }

        self.event_queue
            .lock()
            .expect("Could not get lock on event queue")
            .push(e);
    }

    /// Returns whether the work_queue and event_queue are BOTH empty
    ///
    /// # Returns
    /// * `Result<bool, ConstellationError>` - The result will always contain
    /// True if both queues are empty, otherwise a ConstellationError will be
    /// returned.
    fn done(&mut self) -> Result<bool, ConstellationError> {
        if self.work_queue.lock().unwrap().is_empty() &&
            self.event_queue.lock().unwrap().is_empty() {
            return Ok(true);
        }

        Err(ConstellationError)
    }

    fn identifier(&mut self) -> ConstellationIdentifier {
        self.identifier
            .lock()
            .expect("Could not get lock on ConstellationIdentifier")
            .clone()
    }

    fn is_master(&mut self) -> Result<bool, ConstellationError> {
        if mpi_info::rank(&self.universe) == 0 {
            Ok(true)
        } else {
            Ok(false)
        }
    }

    fn nodes(&mut self) -> i32 {
        self.nodes
    }

    fn generate_identifier(&mut self) -> ConstellationIdentifier {
        // Check if there is a multithreaded Constellation running
        if self.parent.is_none() {
            // Has no parent, this is the top-level instance of Constellation
            return ConstellationIdentifier::new(&self.universe);
        }

        // Call parent method ConstellationIdentifier
        self.parent
            .as_mut()
            .expect("No parent available")
            .lock()
            .unwrap()
            .generate_identifier()
    }
}

impl InnerConstellation {
    pub fn new(
        work_queue: Arc<Mutex<deque::Injector<Box<dyn ActivityWrapperTrait>>>>,
        event_queue: Arc<Mutex<deque::Injector<Box<Event>>>>,
        config: &Box<ConstellationConfiguration>,
    ) -> InnerConstellation {
        let id = Arc::new(Mutex::new(ConstellationIdentifier::new_empty()));
        let mut new_const = InnerConstellation {
            identifier: id,
            universe: mpi::initialize().unwrap(),
            debug: config.debug,
            nodes: config.number_of_nodes,
            work_queue,
            event_queue,
            parent: None,
        };
        new_const.identifier = Arc::new(Mutex::new(new_const.generate_identifier()));

        new_const
    }

    pub fn set_parent(&mut self, parent: Arc<Mutex<dyn ConstellationTrait>>) {
        self.parent = Some(parent.clone());
    }
}
