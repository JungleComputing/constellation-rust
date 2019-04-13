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
use hashbrown::HashMap;
use crate::implementation::thread_helper::ThreadHelper;
use crate::implementation::event_queue::EventQueue;

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
/// * `debug` - Bool indicating whether to print debug messages
/// * `nodes` - Number of nodes in running constellation instance
/// * `context_vec` - Vector of contexts, indicating which activities to execute
/// on this thread
/// * `executor` - The thread actually processing submitted activities
/// * `multi_threaded` - used to indicate whether this instance is running on
/// multiple threads or not. If yes, the suspended queues will be linked with
/// the parent instance.
/// * `work_queue` - WorkQueue used to share activities with the executor thread
/// * `work_queue_suspended` - Work queue containing data which gets suspended
/// by thread
/// * `work_queue_wrong_context` - Work queue containing all activities which have
/// context not existing locally
/// * `work_queue_parent` - Queue used to push work to parent when the regular
/// work_queue is full.
/// * `event_queue` - Queue used to share events with the executor thread
pub struct InnerConstellation {
    identifier: Arc<Mutex<ConstellationIdentifier>>,
    debug: bool,
    nodes: i32,
    context_vec: ContextVec,
    executor: Option<ThreadHandler>,
    multi_threaded: bool,
    parent: Option<ThreadHelper>,
    thread_id: i32,
    pub work_queue: Arc<Mutex<HashMap<ActivityIdentifier, Box<dyn ActivityWrapperTrait>>>>,
    pub work_suspended: Arc<Mutex<HashMap<ActivityIdentifier, Box<dyn ActivityWrapperTrait>>>>,
    pub event_queue: Arc<Mutex<EventQueue>>,
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
            info!("Submitting activity with id: {}", &activity_id);
        }

        if !self.multi_threaded {
            self.work_queue.lock().unwrap().insert(activity_id.clone(), activity_wrapper);
            return activity_id;
        }

        self.parent.as_mut().expect(
            "Found no parent, make sure to set a ThreadHandler"
        ).submit(activity_wrapper);

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

        let aid = e.get_dst();

        // Running single threaded instance
        if !self.multi_threaded {
            self.event_queue.lock().unwrap().insert(aid, e);
            return;
        }

        // Check if we already have the corresponding activity
        let mut exists =  self.work_queue.lock().unwrap().contains_key(&aid);
        if exists {
            self.event_queue.lock().unwrap().insert(aid, e);
            return;
        }

        // Check if we have it in the suspended queue
        exists = self.work_suspended.lock().unwrap().contains_key(&aid);
        if exists {
            self.event_queue.lock().unwrap().insert(aid, e);
            return;
        }

        // Let parent deal with event, perhaps some other thread has the
        // activity
        self.parent.as_mut().expect(
            "No existing parent, make sure to set a ThreadHandler"
        ).send(e);
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
                let (w, w_s) = (self.work_queue.lock().unwrap().len(), self.work_suspended.lock().unwrap().len());
                warn!("Found work left in thread: {}, work_queue len: {}, work_suspended len: {}", self.thread_id, w, w_s);
                return Ok(false);
            }
            _ => (),
        }

        // Shut down thread
        let handler = self.executor.as_ref().unwrap();
        handler
            .sender
            .send(true)
            .expect("Failed to send signal to executor");

        let time = time::Duration::from_secs(100);
        if self.debug {
            info!("Waiting for {}s for executor thread with id: {} to shut down", 100, self.identifier.lock().unwrap());
        }

        if let Ok(r) = handler.receiver.recv_timeout(time) {
            if !r {
                warn!("Executor thread signals that there is work left");
                let (w, w_s) = (self.work_queue.lock().unwrap().len(), self.work_suspended.lock().unwrap().len());
                warn!("Work in thread: {}, work_queue len: {}, work_suspended len: {}", self.thread_id, w, w_s);
                return Ok(false);
            }
        } else {
            warn!("Timeout waiting for the executor thread to shutdown, something is wrong");
            return Err(ConstellationError);
        }

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
        unimplemented!();
    }
}

impl InnerConstellation {
    pub fn new(config: &Box<ConstellationConfiguration>, universe: &Universe, activity_counter: Arc<Mutex<u64>>, thread_id: i32) -> InnerConstellation {
        InnerConstellation {
            identifier: Arc::new(Mutex::new(ConstellationIdentifier::new(universe, activity_counter, thread_id))),
            debug: config.debug,
            nodes: config.number_of_nodes,
            context_vec: config.context_vec.clone(),
            executor: None,
            multi_threaded: false,
            parent: None,
            thread_id,
            work_queue: Arc::new(Mutex::new(HashMap::new())),
            work_suspended: Arc::new(Mutex::new(HashMap::new())),
            event_queue: Arc::from(Mutex::from(EventQueue::new())),
        }
    }

    pub fn new_multithreaded(config: &Box<ConstellationConfiguration>,
                             identifier: Arc<Mutex<ConstellationIdentifier>>,
                             parent: ThreadHelper,
                             work_queue: Arc<Mutex<HashMap<ActivityIdentifier, Box<dyn ActivityWrapperTrait>>>>,
                             work_suspended: Arc<Mutex<HashMap<ActivityIdentifier, Box<dyn ActivityWrapperTrait>>>>,
                             event_queue: Arc<Mutex<EventQueue>>,
                             thread_id: i32,

    ) -> InnerConstellation {
        InnerConstellation {
            identifier,
            debug: config.debug,
            nodes: config.number_of_nodes,
            context_vec: config.context_vec.clone(),
            executor: None,
            multi_threaded: true,
            parent: Some(parent),
            thread_id,
            work_queue,
            work_suspended,
            event_queue,
        }
    }


    /// Check if there is work left in the queues
    ///
    /// # Returns
    /// * `bool` - True if there is work in at least one queue, false otherwise
    pub fn work_left(&mut self) -> bool{
        if self.work_queue.lock().unwrap().is_empty()
            && self.work_suspended.lock().unwrap().is_empty()
            && self.event_queue.lock().unwrap().is_empty()
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
    pub fn activate_inner(
        &mut self,
        inner_constellation: Arc<Mutex<Box<dyn ConstellationTrait>>>,
    ){
        let (s, r): (Sender<bool>, Receiver<bool>) = unbounded();
        let (s2, r2): (Sender<bool>, Receiver<bool>) = unbounded();

        let inner_work_queue = self.work_queue.clone();
        let inner_work_suspended = self.work_suspended.clone();
        let inner_event_queue = self.event_queue.clone();
        let id = self.thread_id;

        // Start executor thread, it will keep running until shut down by
        // Constellation
        thread::spawn(move || {
            let mut executor = ExecutorThread::new(
                inner_work_queue,
                inner_work_suspended,
                inner_event_queue,
                inner_constellation,
                r,
                s2,
                id,
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
