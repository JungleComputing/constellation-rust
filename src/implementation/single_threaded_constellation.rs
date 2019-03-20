//! Single threaded implementation of Constellation.
extern crate crossbeam;
extern crate mpi;

use super::super::activity::ActivityTrait;
use super::super::constellation;
use super::super::constellation::ConstellationTrait;
use super::super::constellation_config::ConstellationConfiguration;
use super::super::context::Context;
use super::super::event::Event;
use super::super::implementation::error::ConstellationError;
use super::activity_wrapper::{ActivityWrapper, ActivityWrapperTrait};
use super::constellation_identifier::ConstellationIdentifier;

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

/// A single threaded Constellation instance containing.
///
/// * `identifier` - ConstellationIdentifier used to identify this
/// constellation instance.
/// * `fresh` - Work_queue containing fresh work that anyone can steal.
/// * `stolen` - Work_queue containing stolen work from other Constellation
/// instances.
pub struct SingleThreadConstellation<'a> {
    identifier: ConstellationIdentifier,
    work_queue: Arc<Mutex<deque::Injector<Box<dyn ActivityWrapperTrait>>>>,
    event_queue: Arc<Mutex<deque::Injector<Box<Event>>>>,
    active: bool,
    universe: Universe,
    parent: Option<&'a constellation::ConstellationTrait>,
    executor: Option<ThreadHandler>,
}

impl<'a> constellation::ConstellationTrait for SingleThreadConstellation<'a> {
    fn activate(&mut self) -> Result<bool, ConstellationError> {
        let (sender, receiver): (Sender<i32>, Receiver<i32>) = unbounded();
        let (sender2, receiver2) = (sender.clone(), receiver.clone());

        let inner_work_queue = self.work_queue.clone();
        let inner_event_queue = self.event_queue.clone();

        // Start executor thread, it will keep running untill shut down by
        // Constellation
        let join_handle = thread::spawn(move || {
            // Start checking periodically for work
            let local_work_queue = inner_work_queue.lock().unwrap();
            let local_event_queue = inner_event_queue.lock().unwrap();

            let executor = ExecutorThread::new(
                sender2,
                receiver2,
                local_work_queue,
                local_event_queue);
            executor.run();
        });

        self.executor = Some(ThreadHandler::new(join_handle, sender, receiver));

        self.active = true;

        return Result::Ok(true);
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
    /// * `can_be_stolen` - A boolean indicating whether this activity can be
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
        can_be_stolen: bool,
        expects_events: bool,
    ) -> ActivityIdentifier {
        let activity_wrapper =
            ActivityWrapper::new(&self.identifier, activity, context, can_be_stolen, expects_events);

        let activity_id = activity_wrapper.identifier();

        // Insert ActivityWrapper in injector_queue
        self.work_queue
            .lock()
            .expect("Could not get lock on injector_queue, failed to push activity")
            .push(activity_wrapper);

        activity_id
    }

    fn send(&self, _e: Event) {
        unimplemented!()
    }

    fn done(&self) -> Result<bool, ConstellationError> {
        unimplemented!()
    }

    fn identifier(&self) -> ConstellationIdentifier {
        self.identifier.clone()
    }

    fn is_master(&self) -> Result<bool, ConstellationError> {
        if self.rank() == 0 {
            Ok(true)
        } else {
            Ok(false)
        }
    }

    fn nodes(&self) -> i32 {
        self.world().size()
    }

    /// Generate a unique ConstellationIdentifier by recursively calling this
    /// method on all possible parent ConstellationTrait instances
    ///
    /// # Returns
    /// * `ConstellationIdentifier` - A unique ConstellationIdentifier
    fn generate_identifier(&self) -> ConstellationIdentifier {
        // Check if there is a multithreaded Constellation running
        if self.parent.is_none() {
            // Has no parent, this is the top-level instance of Constellation
            return ConstellationIdentifier::new(&self.universe);
        }

        // Call parent method ConstellationIdentifier
        self.parent.expect(
            "No parent available"
        ).generate_identifier()

    }
}

impl<'a> SingleThreadConstellation<'a> {
    /// Create a new single threaded constellation instance, initializing
    /// ConstellationID, NodeMapping and relevant queues
    ///
    /// # Arguments
    /// * `config` - A boxed ConstellationConfiguration
    ///
    /// # Returns
    /// * `SingleThreadedConstellation` - New single threaded Constellation
    /// instance
    pub fn new(_config: Box<ConstellationConfiguration>) -> SingleThreadConstellation<'a> {
        let const_id = ConstellationIdentifier::new_empty();
        let mut new_const = SingleThreadConstellation {
            identifier: const_id,
            work_queue: Arc::new(Mutex::new(deque::Injector::new())),
            event_queue: Arc::new(Mutex::new(deque::Injector::new())),
            active: false,
            universe: mpi::initialize().unwrap(),
            parent: None,
            executor: None,
        };

        new_const.identifier = new_const.generate_identifier();

        new_const
    }

    fn rank(&self) -> i32 {
        self.world().rank()
    }

    fn world(&self) -> SystemCommunicator {
        self.universe.world()
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
/// instance.
///
/// * `sender` - Sender channel to send data to the Constellation instance.
/// * `receiver` - Receiver channel to receive data from the Constellation instance.
/// * `work_queue` - Shared queue with Constellation instance, used to grab
/// work when available.
/// * `event_queue` - Shared queue for events containing data, executor will
/// check this queue whenever if is expecting events
struct ExecutorThread<'a> {
    sender: Sender<i32>,
    receiver: Receiver<i32>,
    work_queue: MutexGuard<'a, deque::Injector<Box<dyn ActivityWrapperTrait>>>,
    event_queue: MutexGuard<'a, deque::Injector<Box<Event>>>,
}

impl<'a> ExecutorThread<'a> {
    fn new(
        sender: Sender<i32>,
        receiver: Receiver<i32>,
        work_queue: MutexGuard<'a, deque::Injector<Box<dyn ActivityWrapperTrait>>>,
        event_queue: MutexGuard<'a, deque::Injector<Box<Event>>>,
    ) -> ExecutorThread<'a> {
        ExecutorThread {
            sender,
            receiver,
            work_queue,
            event_queue,
        }
    }

    fn check_for_work(&self) -> Option<Box<dyn ActivityWrapperTrait>> {
        // Steal work from shared activity queue, if available
        if let Steal::Success(activity) = self.work_queue.steal() {
            return Some(activity as Box<dyn ActivityWrapperTrait>);
        }

        None
    }

    fn process_work(&self, activity: Box<dyn ActivityWrapperTrait>) {
        println!("From inside thread: {}", activity.identifier().to_string());
    }

    fn run(&self) {
        let two_seconds = time::Duration::from_secs(10);
        thread::sleep(two_seconds);

        for _ in 0..1 {
            let work = self.check_for_work();
            match work {
                Some(x) => self.process_work(x),
                None => (),
            }
        }
    }
}
