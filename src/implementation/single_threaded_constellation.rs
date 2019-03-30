//! Single threaded implementation of Constellation.
extern crate crossbeam;
extern crate mpi;

use super::super::activity::ActivityTrait;
use super::super::constellation::ConstellationTrait;
use super::super::constellation_config::ConstellationConfiguration;
use super::super::context::Context;
use super::super::event::Event;
use super::super::implementation::error::ConstellationError;
use super::activity_wrapper::{ActivityWrapper, ActivityWrapperTrait};
use super::constellation_identifier::ConstellationIdentifier;
use super::inner_constellation::InnerConstellation;

use mpi::environment::Universe;
use mpi::topology::Communicator;
use mpi::topology::SystemCommunicator;
use std::thread;
use std::thread::JoinHandle;

use crossbeam::crossbeam_channel::{unbounded, Receiver, Sender};
use crossbeam::deque;
use crossbeam::deque::Steal;
use std::sync::MutexGuard;
use std::sync::{Arc, Mutex};
use std::time;
use crate::activity_identifier::ActivityIdentifier;
use std::any::Any;

/// A single threaded Constellation initializer, it creates an executor thread
/// and a InnerConstellation object. The inner_constellation contains all
/// logic related to Constellation (such as submitting activities etc).
/// The only purpose of this wrapper is to initialize both threads and share
/// the references between them.
pub struct SingleThreadConstellation {
    executor: Option<ThreadHandler>,
    inner_constellation: Arc<Mutex<Box<dyn ConstellationTrait>>>
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
        let (sender, receiver): (Sender<i32>, Receiver<i32>) = unbounded();
        let (sender2, receiver2) = (sender.clone(), receiver.clone());

        let mut inner_work_queue: Arc<Mutex<deque::Injector<Box<dyn ActivityWrapperTrait>>>>;
        let mut inner_event_queue: Arc<Mutex<deque::Injector<Box<Event>>>>;

        if let Some(inner) = self.inner_constellation.lock().unwrap().downcast_ref::<InnerConstellation>(){
            inner_work_queue = inner.work_queue.clone();
            inner_event_queue = inner.event_queue.clone();
        } else {
            panic!("Something went wrong when cloning the work and event queue")
        };

        let inner_constellation =
            self.inner_constellation.clone();

        // Start executor thread, it will keep running untill shut down by
        // Constellation
        let join_handle = thread::spawn(move || {
            // Start checking periodically for work
            let local_work_queue = inner_work_queue;
            let local_event_queue = inner_event_queue;

            let mut executor = ExecutorThread::new(
                sender2,
                receiver2,
                local_work_queue,
                local_event_queue,
                inner_constellation,
                );
            executor.run();
        });

        self.executor = Some(ThreadHandler::new(join_handle, sender, receiver));

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
        activity: &Arc<Mutex<dyn ActivityTrait>>,
        context: &Context,
        may_be_stolen: bool,
        expects_events: bool,
    ) -> ActivityIdentifier {
        self.inner_constellation.lock().unwrap().submit(activity,
                                                        context,
                                                        may_be_stolen,
                                                        expects_events)
    }

    /// Perform a send operation with the event specified as argument
    ///
    /// # Arguments
    /// * `e` - Event to send
    fn send(&mut self, e: Event) {
        self.inner_constellation.lock().unwrap().send(e);
    }

    /// Signal Constellation that it is done, perform a graceful shutdown
    ///
    /// # Returns
    /// * `Result<bool, ConstellationError>` - Result type containing true if
    /// it could succesfully shutdown, false otherwise.
    /// Upon error a ConstellationError is returned
    fn done(&mut self) -> Result<bool, ConstellationError> {
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
        self.inner_constellation.lock().unwrap().generate_identifier()
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
    pub fn new(_config: Box<ConstellationConfiguration>) -> SingleThreadConstellation {
        SingleThreadConstellation {
            executor: None,
            inner_constellation: Arc::new(Mutex::new(Box::new(
                InnerConstellation::new(Arc::new(Mutex::new(deque::Injector::new())),
                                        Arc::new(Mutex::new(deque::Injector::new()))
                )))),
        }
    }
}

/// struct holding necessary data structures needed for communication between
/// the executor thread and SingleThreadedConstellation
///
/// * `join_handle` - The handle returned when creating the executor thread
/// * `sender` - Sender channel used for sending data to the executor
/// * `receiver` - Receiver channel used for receiving data from the executor
struct ThreadHandler {
    join_handle: JoinHandle<()>,
    sender: Sender<i32>,
    receiver: Receiver<i32>,
}

impl ThreadHandler {
    fn new(
        join_handle: JoinHandle<()>,
        sender: Sender<i32>,
        receiver: Receiver<i32>,
    ) -> ThreadHandler {
        ThreadHandler {
            join_handle,
            sender,
            receiver,
        }
    }
}

/// Executor thread, runs in a separate thread and is in charge of executing
/// activities. It will periodically check for work in the Constellation
/// instance using it's shared queues.
///
/// * `sender` - Sender channel to send data to the Constellation instance.
/// * `receiver` - Receiver channel to receive data from the Constellation instance.
/// * `work_queue` - Shared queue with Constellation instance, used to grab
/// work when available.
/// * `event_queue` - Shared queue for events containing data, executor will
/// check this queue whenever if is expecting events
/// * `constellation` - A reference to the InnerConstellation instance, required
/// by the functions in the activities executed
struct ExecutorThread {
    sender: Sender<i32>,
    receiver: Receiver<i32>,
    work_queue: Arc<Mutex<deque::Injector<Box<dyn ActivityWrapperTrait>>>>,
    event_queue: Arc<Mutex<deque::Injector<Box<Event>>>>,
    local: deque::Worker<Box<dyn ActivityWrapperTrait>>,
    constellation: Arc<Mutex<Box<dyn ConstellationTrait>>>,
}

impl ExecutorThread {
    /// Create a new ExecutorThread
    ///
    /// * `ExecutorThread` - New executor thread
    fn new(
        sender: Sender<i32>,
        receiver: Receiver<i32>,
        work_queue: Arc<Mutex<deque::Injector<Box<dyn ActivityWrapperTrait>>>>,
        event_queue: Arc<Mutex<deque::Injector<Box<Event>>>>,
        constellation: Arc<Mutex<Box<dyn ConstellationTrait>>>,
    ) -> ExecutorThread{
        ExecutorThread {
            sender,
            receiver,
            work_queue,
            event_queue,
            local: deque::Worker::new_fifo(),
            constellation,
        }
    }

    /// Tries to steal a batch of work from the shared work_queue. If there is
    /// work, it will return one of the stolen jobs, which is to be
    /// executed immediately.
    ///
    /// # Returns
    /// * `Option<Box<dyn ActivityWrapperTrait>>` - If there is work, it will
    /// pop one job from the local queue and return that wrapped in Some(..)
    fn check_for_work(&mut self) -> Option<Box<dyn ActivityWrapperTrait>> {
        // Steal work from shared activity queue, if available
        if let Steal::Success(activity) = self.work_queue.lock().unwrap()
            .steal_batch_and_pop(&self.local) {
            return Some(activity);
        }

        None
    }

    /// Process a stolen activity, this is the main function of executing a
    /// activity.
    ///
    /// ADD MORE WHEN FURTHER
    ///
    /// # Arguments
    /// * `activity` - A boxed activity to perform work on.
    fn process_work(&mut self, mut activity: Box<dyn ActivityWrapperTrait>) {
        self.initialize(activity);
    }

    /// Call initialize on the activity
    fn initialize(&mut self, mut activity: Box<dyn ActivityWrapperTrait>){
        activity.initialize(self.constellation.clone());
    }

    /// This will startup the thread, periodically check for work forever or
    /// if shut down from InnerConstellation/SingleThreadedConstellation.
    fn run(&mut self) {
        let time = time::Duration::from_secs(1);
        thread::sleep(time);

        for _ in 0..20 {
            let mut work = self.check_for_work();
            match work {
                Some(x) => self.process_work(x),
                None => (),
            }
        }
    }
}
