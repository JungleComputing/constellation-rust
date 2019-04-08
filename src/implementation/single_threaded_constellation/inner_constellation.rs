extern crate crossbeam;
extern crate mpi;

use std::sync::{Arc, Mutex};

use crate::activity::ActivityTrait;
use crate::activity_identifier::ActivityIdentifier;
use crate::constellation::ConstellationTrait;
use crate::constellation_config::ConstellationConfiguration;
use crate::constellation_identifier::ConstellationIdentifier;
use crate::context::{Context, ContextVec};
use crate::event::Event;
use crate::implementation::activity_wrapper::{ActivityWrapper, ActivityWrapperTrait};
use crate::implementation::error::ConstellationError;
use crate::implementation::single_threaded_constellation::executor_thread::ExecutorThread;

use crossbeam::deque;
use crossbeam::{unbounded, Receiver, Sender};
use mpi::environment::Universe;
use std::thread;
use std::time;

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
/// * `work_queue` - Queue used to share activities with the executor thread
/// * `work_queue_remote` - Work queue containing all activities which have
/// context not existing locally
/// * `event_queue` - Queue used to share events with the executor thread
/// * `parent` - Possible parent constellation instance, used in multithreading
pub struct InnerConstellation {
    identifier: Arc<Mutex<ConstellationIdentifier>>,
    debug: bool,
    nodes: i32,
    context_vec: ContextVec,
    executor: Option<ThreadHandler>,
    pub work_queue: Arc<Mutex<deque::Injector<Box<dyn ActivityWrapperTrait>>>>,
    pub work_queue_remote: Arc<Mutex<deque::Injector<Box<dyn ActivityWrapperTrait>>>>,
    pub event_queue: Arc<Mutex<deque::Injector<Box<Event>>>>,
    pub parent: Option<Arc<Mutex<Box<dyn ConstellationTrait>>>>,
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

        // Check context
        if self.context_vec.contains(context) {
            // Insert ActivityWrapper in injector_queue
            self.work_queue
                .lock()
                .expect("Could not get lock on injector_queue, failed to push activity")
                .push(activity_wrapper);
        } else {
            // Let multithreaded constellation handle this activity
            self.work_queue_remote
                .lock()
                .expect("Could not get lock on remote work queue")
                .push(activity_wrapper);
        }

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
        // Check if we still have activities running
        match self.work_left() {
            true => {
                return Err(ConstellationError);
            }
            _ => (),
        }

        // Shut down thread
        let handler = self.executor.as_ref().unwrap();
        handler
            .sender
            .send(true)
            .expect("Failed to send signal to executor");

        let time = time::Duration::from_secs(10);
        if self.debug {
            info!("Waiting for {}s for executor thread with id: {} to shut down", 10, self.identifier.lock().unwrap());
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

    fn identifier(&mut self) -> ConstellationIdentifier {
        self.identifier
            .lock()
            .expect("Could not get lock on ConstellationIdentifier")
            .clone()
    }

    fn is_master(&mut self) -> Result<bool, ConstellationError> {
        panic!("This should never be called on the inner constellation instance");
    }

    fn nodes(&mut self) -> i32 {
        self.nodes
    }

    fn set_parent(&mut self, parent: Arc<Mutex<Box<dyn ConstellationTrait>>>) {
        self.parent = Some(parent.clone());
    }
}

impl InnerConstellation {
    pub fn new(config: &Box<ConstellationConfiguration>, universe: &Universe, thread_id: i32) -> InnerConstellation {
        InnerConstellation {
            identifier: Arc::new(Mutex::new(ConstellationIdentifier::new(universe, thread_id))),
            debug: config.debug,
            nodes: config.number_of_nodes,
            context_vec: config.context_vec.clone(),
            executor: None,
            work_queue: Arc::from(Mutex::from(deque::Injector::new())),
            work_queue_remote: Arc::new(Mutex::new(deque::Injector::new())),
            event_queue: Arc::from(Mutex::from(deque::Injector::new())),
            parent: None,
        }
    }

    pub fn new_multithreaded(config: &Box<ConstellationConfiguration>, _: i32) -> InnerConstellation {
        InnerConstellation {
            identifier: Arc::new(Mutex::new(ConstellationIdentifier::new_empty())),
            debug: config.debug,
            nodes: config.number_of_nodes,
            context_vec: config.context_vec.clone(),
            executor: None,
            work_queue: Arc::from(Mutex::from(deque::Injector::new())),
            work_queue_remote: Arc::new(Mutex::new(deque::Injector::new())),
            event_queue: Arc::from(Mutex::from(deque::Injector::new())),
            parent: None, // NO IDENTIFIER SET HERE
        }
    }

    /// Check if there is work left in the queues
    ///
    /// # Returns
    /// * `bool` - True if there is work in at least one queue, false otherwise
    pub fn work_left(&mut self) -> bool{
        if self.work_queue.lock().unwrap().is_empty() && self.event_queue.lock().unwrap().is_empty()
        {
            return false;
        }

        return true;
    }

    /// Method that creates the executor thread and activates InnerConstellation
    ///
    /// # Arguments
    /// * `inner_constellation` - An Arc<Mutex<..>> reference to the
    /// constellation instance on which THIS method was called.
    /// * `work_queue` - The work_queue which will be used for sharing
    /// activities between ExecutorThread and InnerConstellation
    /// * `event_queue` - Shared event queue between ExecutorThread and
    /// InnerConstellation
    pub fn activate_inner(
        &mut self,
        inner_constellation: Arc<Mutex<Box<dyn ConstellationTrait>>>,
        work_queue: Arc<Mutex<deque::Injector<Box<dyn ActivityWrapperTrait>>>>,
        event_queue: Arc<Mutex<deque::Injector<Box<Event>>>>,
    ){
        let (s, r): (Sender<bool>, Receiver<bool>) = unbounded();
        let (s2, r2): (Sender<bool>, Receiver<bool>) = unbounded();

        // Start executor thread, it will keep running until shut down by
        // Constellation
        thread::spawn(move || {
            // Start checking periodically for work
            let local_work_queue = work_queue;
            let local_event_queue = event_queue;

            let mut executor = ExecutorThread::new(
                local_work_queue,
                local_event_queue,
                inner_constellation,
                r,
                s2,
            );
            executor.run();
        });

        self.executor = Some(ThreadHandler::new(r2, s));
    }
}

/// struct holding necessary data structures needed for communication between
/// the executor thread and InnerConstellation
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
