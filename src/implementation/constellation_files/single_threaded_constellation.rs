//! Single threaded implementation of Constellation.
extern crate crossbeam;
extern crate mpi;

use super::inner_constellation::InnerConstellation;
use crate::implementation::communication::mpi_info;
use crate::implementation::constellation_identifier::ConstellationIdentifier;
use crate::{
    ActivityIdentifier, ActivityTrait, ConstellationConfiguration, ConstellationError,
    ConstellationTrait, Context, Event,
};
use mpi::environment::Universe;

use std::sync::{Arc, Mutex};

/// A single threaded Constellation initializer, it creates an executor thread
/// and a InnerConstellation object. The inner_constellation contains all
/// logic related to Constellation (such as submitting activities etc).
/// The only purpose of this wrapper is to initialize both threads and share
/// the references between them.
///
/// # Members
/// * `inner_constellation` - An Arc<Mutex<..>> reference to an
/// InnerConstellation struct. This struct is shared with the executor thread
/// in order for the executor to be able to submit/send events using the
/// Constellation trait
/// * `universe` - MPI Universe struct
/// * `debug` - boolean indicating whether to display debug messages or not
pub struct SingleThreadConstellation {
    inner_constellation: Arc<Mutex<Box<dyn ConstellationTrait>>>,
    universe: Universe,
    debug: bool,
}

impl ConstellationTrait for SingleThreadConstellation {
    /// Activate the Constellation instance
    ///
    /// This will setup the ExecutorThread and the InnerConstellation object,
    /// and share necessary references between them.
    ///
    /// # Returns
    /// * `Result<bool, ConstellationError>` - A Result type containing a
    /// boolean which will have the value true if this is the master thread and
    /// false otherwise.
    ///
    /// Upon failure a ConstellationError will be returned
    fn activate(&mut self) -> Result<bool, ConstellationError> {
        if self.is_master().unwrap() {
            if self.debug {
                info!("Activating Single Threaded Constellation");
            }
            self.inner_constellation
                .lock()
                .unwrap()
                .downcast_mut::<InnerConstellation>()
                .unwrap()
                .activate_inner(self.inner_constellation.clone());

            return Ok(true);
        }
        return Ok(false);
    }

    /// Submit an activity to Constellation. Internally it will wrap the new
    /// activity inside an ActivityWrapper, which will generate a new unique
    /// activity ID.
    ///
    /// The wrapper will be pushed to `self.injector_queue`, where it can be
    /// stolen by the `executor` thread.
    ///
    /// # Arguments
    /// * `activity` - A reference to an activity implementing the ActivityTrait.
    /// The activity must be inside an Arc<Mutex<..>>, in order to work with
    /// thread safety.
    /// * `context` - A reference to the context created for this activity
    /// * `may_be_stolen` - A boolean indicating whether this activity can be
    /// stolen or not.
    /// * `expects_events` - A boolean indicating whether this activity expects
    /// events or not.
    ///
    /// # Returns
    /// * `ActivityIdentifier` - The generated Activity Identifier for
    /// this Activity
    fn submit(
        &mut self,
        activity: Arc<Mutex<dyn ActivityTrait>>,
        context: &Context,
        may_be_stolen: bool,
        expects_events: bool,
    ) -> ActivityIdentifier {
        self.inner_constellation.lock().unwrap().submit(
            activity,
            context,
            may_be_stolen,
            expects_events,
        )
    }

    /// Perform a send operation with the event specified as argument
    ///
    /// # Arguments
    /// * `e` - Event to send
    fn send(&mut self, e: Box<Event>) {
        self.inner_constellation.lock().unwrap().send(e);
    }

    /// Signal Constellation that it is done, perform a graceful shutdown
    ///
    /// # Returns
    /// * `Result<bool, ConstellationError>` - Result type containing true if
    /// it could successfully shutdown, false otherwise.
    ///
    /// Upon error a ConstellationError is returned
    fn done(&mut self) -> Result<bool, ConstellationError> {
        if self.debug {
            info!("Attempting to shut down Constellation gracefully");
        }

        self.inner_constellation.lock().unwrap().done()
    }

    /// Retrieve an identifier for this Constellation instance
    ///
    /// # Returns
    /// * `ConstellationIdentifier` - Identifier for this Constellation instance
    fn identifier(&mut self) -> ConstellationIdentifier {
        self.inner_constellation.lock().unwrap().identifier()
    }

    /// Retrieve if THIS process is the master, used for leader election.
    /// Only ONE process will return true, the rest will return false.
    ///
    /// # Returns
    /// * `Result<bool, ConstellationError` - Result type with the value true if
    /// this process is the leader, false otherwise.
    /// Will return ConstellationError if something went wrong.
    fn is_master(&self) -> Result<bool, ConstellationError> {
        if mpi_info::rank(&self.universe) == 0 {
            Ok(true)
        } else {
            Ok(false)
        }
    }

    /// Return the total number of nodes in the Constellation instance
    ///
    /// # Returns
    /// * `i32` - Number of nodes
    fn nodes(&mut self) -> i32 {
        self.inner_constellation.lock().unwrap().nodes()
    }
}

impl SingleThreadConstellation {
    /// Create a new single threaded constellation instance, initializing
    /// ConstellationID, NodeMapping and relevant queues
    ///
    /// # Arguments
    /// * `config` - A boxed ConstellationConfiguration
    ///
    /// # Returns
    /// * `SingleThreadedConstellation` - New single threaded Constellation
    /// instance
    pub fn new(config: Box<ConstellationConfiguration>) -> SingleThreadConstellation {
        let universe = mpi::initialize().unwrap();

        SingleThreadConstellation {
            inner_constellation: Arc::new(Mutex::new(Box::new(InnerConstellation::new(
                &config,
                &universe,
                Arc::new(Mutex::new(0)),
                0,
            )))),
            universe,
            debug: config.debug,
        }
    }
}
