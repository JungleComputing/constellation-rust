///! Module for handling:
///! - Thread synchronization
///! - Load balancing
///! - Inter-node activity stealing
///! - Distributing activities submitted by threads
///! - Making sure Events get sent to the correct thread (holding the
///! target activity)
///!
///! All logic regarding the above stated capabilities are handled in this
///! module the MultiThreadedConstellation is simply a wrapper class making sure
///! that all Arc<Mutex<..>> variables are correctly created for each thread.
///!
///! The `run` method should be started with a new thread, ìt will periodically
///! check threads for suspended activities and events to distribute evenly
///! across all threads.

use crate::implementation::activity_wrapper::{ActivityWrapper, ActivityWrapperTrait};
use crate::implementation::constellation_identifier::ConstellationIdentifier;
use crate::implementation::event_queue::EventQueue;
use crate::{
    ActivityIdentifier, ActivityTrait, ConstellationError, ConstellationTrait, Context, Event,
};

use std::sync::{Arc, Mutex};
use std::thread;
use std::time;

use crossbeam::{deque, deque::Steal, Receiver, Sender};
use hashbrown::HashMap;

/// Struct holding all queues related to one single thread.
///
/// # Members
/// * `const_id` - Constellation identifier with a thread number
/// * `activities` - Main activity HashMap, all activities are submitted to this
/// struct
/// * `activities_suspended` - Suspended activities
/// * `event_queue` - Event queue
#[derive(Clone)]
pub struct ExecutorQueues {
    pub const_id: Arc<Mutex<ConstellationIdentifier>>,
    pub activities: Arc<Mutex<HashMap<ActivityIdentifier, Box<dyn ActivityWrapperTrait>>>>,
    pub activities_suspended:
    Arc<Mutex<HashMap<ActivityIdentifier, Box<dyn ActivityWrapperTrait>>>>,
    pub event_queue: Arc<Mutex<EventQueue>>,
}

impl ExecutorQueues {
    pub fn new(constellation_identifier: Arc<Mutex<ConstellationIdentifier>>) -> ExecutorQueues {
        ExecutorQueues {
            const_id: constellation_identifier,
            activities: Arc::new(Mutex::new(HashMap::new())),
            activities_suspended: Arc::new(Mutex::new(HashMap::new())),
            event_queue: Arc::new(Mutex::new(EventQueue::new())),
        }
    }
}

/// Structure holding a shared activity and event queue, which is used to pass
/// activities and events from the thread to the thread_handler
///
/// # Members
/// * `activities` - Reference to an Injector queue containing activities
/// * `events` - Reference to an Injector queue containing events
#[derive(Clone)]
pub struct ThreadHelper {
    activities: Arc<Mutex<deque::Injector<Box<dyn ActivityWrapperTrait>>>>,
    events: Arc<Mutex<deque::Injector<Box<Event>>>>,
}

impl ThreadHelper {
    pub fn new(
        activities: Arc<Mutex<deque::Injector<Box<dyn ActivityWrapperTrait>>>>,
        events: Arc<Mutex<deque::Injector<Box<Event>>>>,
    ) -> ThreadHelper {
        ThreadHelper { activities, events }
    }

    /// Can be called from inside the InnerConstellation to share with
    /// MultiThreadHelper
    pub fn submit(&mut self, activity_wrapper: Box<ActivityWrapper>) {
        self.activities.lock().unwrap().push(activity_wrapper);
    }

    /// Can be called from inside the InnerConstellation to share with
    /// MultiThreadHelper
    pub fn send(&mut self, e: Box<Event>) {
        self.events.lock().unwrap().push(e);
    }
}

/// Structure holding all thread information, references to queues inside
/// threads for pushing new work and the queues used to retrieve work/events
/// when a thread submits them.
///
/// The run method is used to periodically check the queues in ThreadHelper for
/// activities/events passed on from inside threads.
///
/// # Members
/// * `threads` - Vector containing each threads specific InnerConstellation
/// instance as well as references to all their queues
/// * `time_between_checks` - The time to wait between checking threads for
/// activities and events that have been submitted. OPTIMIZATION: This can be
/// fine-tuned for performance depending on application, for example: if an
/// application has very compute heavy activities but submits few new ones,
/// this time should be larger to avoid checking (getting a mutex) on queues
/// too often.
/// * `debug` - Boolean, indicates whether to display debug messages or not
/// * `activities_from_threads` - Activities passed on from threads,
/// should be shared with the ThreadHelper
/// * `events_from_threads` - Events passed on from threads, should be shared
/// with the ThreadHelper
/// * `local_events` - Stores events which have no matching activity on this
/// node
#[derive(Clone)]
pub struct MultiThreadHelper {
    pub threads: Vec<(Arc<Mutex<Box<dyn ConstellationTrait>>>, ExecutorQueues)>,
    time_between_steals: time::Duration,
    debug: bool,
    activities_from_threads: Arc<Mutex<deque::Injector<Box<dyn ActivityWrapperTrait>>>>,
    events_from_threads: Arc<Mutex<deque::Injector<Box<Event>>>>,
    local_events: Arc<Mutex<EventQueue>>,
}

impl MultiThreadHelper {
    /// Create new, clean instance
    ///
    /// # Arguments
    /// * `debug` - Boolean indicating whether to print debug messages or not
    /// * `activities_from_threads` - Activities passed on from threads,
    /// should be shared with the ThreadHelper
    /// * `events_from_threads` - Events passed on from threads, should be shared
    /// with the ThreadHelper
    pub fn new(
        debug: bool,
        activities_from_threads: Arc<Mutex<deque::Injector<Box<dyn ActivityWrapperTrait>>>>,
        events_from_threads: Arc<Mutex<deque::Injector<Box<Event>>>>,
        time_between_steals: u64,
    ) -> MultiThreadHelper {
        MultiThreadHelper {
            threads: Vec::new(),
            time_between_steals: time::Duration::from_micros(time_between_steals),
            debug,
            activities_from_threads,
            events_from_threads,
            local_events: Arc::new(Mutex::new(EventQueue::new())),
        }
    }

    /// Push new thread
    ///
    /// # Arguments
    /// * `Executor_queues` - ExecutorQueues type, containing references to all
    /// activiy and event queues for the specifies thread.
    /// * `constellation` - Reference to the InnerConstellation instance
    /// associated with this thread
    pub fn push(
        &mut self,
        executor_queues: ExecutorQueues,
        constellation: Arc<Mutex<Box<dyn ConstellationTrait>>>,
    ) {
        self.threads.push((constellation, executor_queues));
    }

    /// Periodically checks for events from the queues which should be shared
    /// with all threads using the ThreadHelper struct. This should be run
    /// in a separate thread.
    ///
    /// The receiver and sender channels are used in order to communicate when
    /// to shut down this thread.
    ///
    /// # Arguments
    /// * `receiver` - The receiving channel for this thread
    /// * `sender` - The sending channel for this thread
    pub fn run(&mut self, receiver: Receiver<bool>, sender: Sender<bool>) {
        loop {
            // Check for events from threads
            if !self.events_from_threads.lock().unwrap().is_empty() {
                self.handle_thread_events();
            }

            // Check for activities from threads
            if !self.activities_from_threads.lock().unwrap().is_empty() {
                self.handle_thread_activity();
            }

            // Check local events
            self.handle_local_events();

            // Check for signal to shut down
            if let Ok(_) = receiver.try_recv().map(|val| {
                if val {
                    // Signal that we are shutting down
                    sender.send(true).expect(
                        "Failed to send signal to \
                         InnerConstellation from executor thread",
                    );
                    return; // Shutdown thread
                }
            }) {};

            // Sleep for the given time
            thread::sleep(self.time_between_steals);
        }
    }

    /// Submit a new activity to the best suited thread. Internally it will
    /// wrap the new activity inside an ActivityWrapper, which will generate a
    /// new unique activity ID and store other useful information.
    ///
    /// # Arguments
    /// * `activity` - A reference to an activity implementing the ActivityTrait.
    /// The activity must be inside an Arc<Mutex<..>>, in order to work with
    /// thread safety.
    /// * `context` - A reference to the context created for this activity.
    /// * `may_be_stolen` - A boolean indicating whether this activity can be
    /// stolen or not.
    /// * `expects_events` - A boolean indicating whether this activity expects
    /// events or not. Can be used for optimization.
    ///
    /// # Returns
    /// * `ActivityIdentifier` - The generated Activity Identifier for
    /// this Activity
    pub fn submit(
        &mut self,
        activity: Arc<Mutex<ActivityTrait>>,
        context: &Context,
        may_be_stolen: bool,
        expects_events: bool,
    ) -> ActivityIdentifier {
        let index = self.get_thread_with_least_work();

        let thread = &self.threads[index].1;

        let const_id = thread.const_id.clone();

        let activity_wrapper =
            ActivityWrapper::new(const_id, activity, context, may_be_stolen, expects_events);
        let aid = activity_wrapper.activity_identifier().clone();

        if self.debug {
            info!("Submitting activity with ID: {} to thread: {}", &aid, index);
        }

        self.threads[index]
            .1
            .activities
            .lock()
            .unwrap()
            .insert(aid.clone(), activity_wrapper);

        aid
    }

    /// Perform a send operation with the event specified as argument
    ///
    /// # Arguments
    /// * `e` - Event to send
    pub fn send(&mut self, e: Box<Event>) {
        if self.debug {
            info!("Send Event: {} -> {}", e.get_src(), e.get_dst());
        }
        self.distribute_event(e);
    }

    /// (Try) to perform a graceful shutdown of all threads
    ///
    /// # Returns
    /// * `Result<bool, ConstellationError>` - Result type containing true if
    /// it could successfully shutdown all threads, false otherwise.
    ///
    /// Upon error a ConstellationError is returned
    pub fn done(&mut self) -> Result<bool, ConstellationError> {
        for x in 0..self.threads.len() {
            if let Ok(res) = self.threads[x]
                .0
                .lock()
                .expect("Could not get lock on constellation instance")
                .done()
            {
                if !res {
                    return Ok(false);
                }
            } else {
                warn!("Got Error when shutting down thread: {}", x);
                return Err(ConstellationError);
            }
        }

        Ok(true)
    }

    /// Find the thread with the least combined work in it's work queue and
    /// suspended queue.
    ///
    /// # Returns
    /// * `usize` - the index of the thread which has the least work currently.
    fn get_thread_with_least_work(&mut self) -> usize {
        let mut shortest = u64::max_value();
        let mut index = 0;

        for i in 0..self.threads.len() {
            let length = self.threads[i].1.activities.lock().unwrap().len()
                + self.threads[i].1.activities_suspended.lock().unwrap().len();
            if length < shortest as usize {
                index = i;
                shortest = length as u64;
            }
        }

        index
    }

    /// Send an event to the thread containing the target activity. If no such
    /// thread exists, store event locally. Use the `run` method to periodically
    /// search for the activity
    fn distribute_event(&mut self, event: Box<Event>) {
        let key = event.get_dst();

        for i in 0..self.threads.len() {
            let c1 = self.threads[i]
                .1
                .activities
                .lock()
                .unwrap()
                .contains_key(&key);
            let c2 = self.threads[i]
                .1
                .activities_suspended
                .lock()
                .unwrap()
                .contains_key(&key);
            if c1 || c2 {
                self.threads[i]
                    .1
                    .event_queue
                    .lock()
                    .unwrap()
                    .insert(key, event);
                return;
            }
        }

        // Event does not exist in any activity yet, let it sit in our local
        // queue until we find a matching activity. This should in essence only
        // be possible when an event has an invalid destination, or is retrieved
        // from another node, without the matching activity
        self.local_events
            .lock()
            .unwrap()
            .insert(event.get_dst(), event);
    }

    /// Handles all events from threads by looping through the
    /// `self.events_from_threads` queue, stealing all events and distributing
    /// them to the thread which has the corresponding activity.
    fn handle_thread_events(&mut self) {
        loop {
            let event = self.events_from_threads.lock().unwrap().steal();
            match event {
                Steal::Success(e) => {
                    self.distribute_event(e);
                }
                _ => {
                    return;
                }
            }
        }
    }

    /// Insert an activity to the thread which has the least work
    ///
    /// # Arguments
    /// * `activity_trait` - The activity to submit
    fn distribute_activity(&mut self, activity_trait: Box<dyn ActivityWrapperTrait>) {
        let index = self.get_thread_with_least_work();

        let aid = activity_trait.activity_identifier();

        self.threads[index]
            .1
            .activities
            .lock()
            .unwrap()
            .insert(aid.clone(), activity_trait);
    }

    /// Goes through all local events and checks if any thread has the target
    /// activity.
    fn handle_local_events(&mut self) {
        let mut guard = self.local_events.lock().unwrap();
        if guard.is_empty() {
            drop(guard);
            return;
        }

        let mut key = None;

        let mut it = guard.keys().take(1).map(|x| key = Some(x.clone()));
        it.next();

        if !key.is_some() {
            drop(guard);
            return;
        }

        let event = guard.remove(key.unwrap()).unwrap();

        drop(guard);

        self.distribute_event(event);
    }

    /// Handle activities from threads, checks the
    /// `self.activities_from_threads` to find these activities, this struct
    /// should be shared with ALL threads through the ThreadHelper struct.
    fn handle_thread_activity(&mut self) {
        // Load balance activities
        let activity = self.activities_from_threads.lock().unwrap().steal();
        match activity {
            Steal::Success(activity) => {
                self.distribute_activity(activity);
            }
            _ => {}
        }

        // Make sure event goes to correct thread
        let event = self.events_from_threads.lock().unwrap().steal();
        match event {
            Steal::Success(e) => {
                self.distribute_event(e);
            }
            _ => {}
        }
    }
}
