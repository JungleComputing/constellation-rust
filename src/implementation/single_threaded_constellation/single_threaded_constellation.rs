//! Single threaded implementation of Constellation.
extern crate crossbeam;
extern crate mpi;

use std::sync::{Arc, Mutex};
use std::thread;

use super::super::activity_wrapper::ActivityWrapperTrait;
use super::super::error::ConstellationError;
use super::executor_thread::ExecutorThread;
use super::inner_constellation::InnerConstellation;
use crate::activity::ActivityTrait;
use crate::activity_identifier::ActivityIdentifier;
use crate::constellation::ConstellationTrait;
use crate::constellation_config::ConstellationConfiguration;
use crate::constellation_identifier::ConstellationIdentifier;
use crate::context::Context;
use crate::event::Event;

use crossbeam::deque;
use crossbeam::{Receiver, Sender, unbounded};
use std::time;

/// A single threaded Constellation initializer, it creates an executor thread
/// and a InnerConstellation object. The inner_constellation contains all
/// logic related to Constellation (such as submitting activities etc).
/// The only purpose of this wrapper is to initialize both threads and share
/// the references between them.
pub struct SingleThreadConstellation {
    executor: Option<ThreadHandler>,
    inner_constellation: Arc<Mutex<Box<dyn ConstellationTrait>>>,
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
    /// boolean which will ALWAYS have the value true.
    /// Upon failure a ConstellationError will be returned
    fn activate(&mut self) -> Result<bool, ConstellationError> {
        if self.debug {
            info!("Activating Single Threaded Constellation");
        }

        let mut inner_work_queue: Arc<Mutex<deque::Injector<Box<dyn ActivityWrapperTrait>>>>;
        let mut inner_event_queue: Arc<Mutex<deque::Injector<Box<Event>>>>;
        let (s, r): (Sender<bool>, Receiver<bool>) = unbounded();
        let (s2, r2): (Sender<bool>, Receiver<bool>) = unbounded();

        if let Some(inner) = self
            .inner_constellation
            .lock()
            .unwrap()
            .downcast_ref::<InnerConstellation>()
        {
            inner_work_queue = inner.work_queue.clone();
            inner_event_queue = inner.event_queue.clone();
        } else {
            panic!("Something went wrong when cloning the work and event queue")
        };

        let inner_constellation = self.inner_constellation.clone();

        // Start executor thread, it will keep running untill shut down by
        // Constellation
        thread::spawn(move || {
            // Start checking periodically for work
            let local_work_queue = inner_work_queue;
            let local_event_queue = inner_event_queue;

            let mut executor =
                ExecutorThread::new(local_work_queue, local_event_queue, inner_constellation, r, s2);
            executor.run();
        });

        self.executor = Some(ThreadHandler::new(r2, s));

        return Ok(true);
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
    /// it could succesfully shutdown, false otherwise.
    /// Upon error a ConstellationError is returned
    fn done(&mut self) -> Result<bool, ConstellationError> {
        if self.debug {
            info!("Attempting to shut down Constellation gracefully");
        }

        // Check if we still have activities running
        let mut guard = self.inner_constellation.lock().unwrap();

        let inner_result = guard.done();

        match inner_result {
            Err(_) => {
                return inner_result
            },
            _ => ()
        }
        drop(guard);

        // Shut down thread
        let handler = self.executor.as_ref().unwrap();
        handler.sender.send(true).expect(
            "Failed to send signal to executor"
        );

        let time = time::Duration::from_secs(10);
        if self.debug {
            info!("Waiting for {}s for executor thread to shut down", 10);
        }

        if let Ok(r) = handler.receiver.recv_timeout(time) {
            if !r {
                warn!("Executor thread has activities or events to process");
                return Err(ConstellationError);
            }
        } else {
            warn!("Timeout waiting for the executor thread to shutdown");
            return Err(ConstellationError);
        }

        info!("Shutdown successful");
        Ok(true)

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
    fn is_master(&mut self) -> Result<bool, ConstellationError> {
        self.inner_constellation.lock().unwrap().is_master()
    }

    /// Return the total number of nodes in the Constellation instance
    ///
    /// # Returns
    /// * `i32` - Number of nodes
    fn nodes(&mut self) -> i32 {
        self.inner_constellation.lock().unwrap().nodes()
    }

    /// Generate a unique ConstellationIdentifier by recursively calling this
    /// method on all possible parent ConstellationTrait instances
    ///
    /// # Returns
    /// * `ConstellationIdentifier` - A unique ConstellationIdentifier
    fn generate_identifier(&mut self) -> ConstellationIdentifier {
        self.inner_constellation
            .lock()
            .unwrap()
            .generate_identifier()
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
        SingleThreadConstellation {
            executor: None,
            inner_constellation: Arc::new(Mutex::new(Box::new(InnerConstellation::new(
                Arc::new(Mutex::new(deque::Injector::new())),
                Arc::new(Mutex::new(deque::Injector::new())),
                Arc::new(Mutex::new(deque::Injector::new())),
                &config,
            )))),
            debug: config.debug,
        }
    }
}

/// struct holding necessary data structures needed for communication between
/// the executor thread and SingleThreadedConstellation
///
/// * `receiver` - Used to receive signals from executor thread
/// * `sender` - Used to send signal executor thread when ready
/// to shut down gracefully
struct ThreadHandler {
    receiver: Receiver<bool>,
    sender: Sender<bool>,
}

impl ThreadHandler {
    fn new(receiver: Receiver<bool>, sender: Sender<bool>) -> ThreadHandler {
        ThreadHandler { receiver, sender }
    }
}
