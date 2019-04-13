extern crate crossbeam;

use std::sync::{Arc, Mutex};
use std::time;

use super::super::activity_wrapper::ActivityWrapperTrait;
use crate::{activity, activity_identifier, ConstellationTrait, Event};
use crate::activity_identifier::ActivityIdentifier;
use crate::implementation::event_queue::EventQueue;

use crossbeam::deque::Steal;
use crossbeam::{Receiver, Sender};
use hashbrown::HashMap;

// Timeout for trying to steal events from parent before checking suspended
// queues
const SLEEP_TIME: u64 = 100;

/// The executor thread runs in asynchronously and is in charge of executing
/// activities. It will periodically check for work/events in the Constellation
/// instance using it's shared queues. Closely coupled to inner_constellation.
///
/// When activities are re-activated with events after being suspended, they
/// will start by immediately calling the process method (possibly again).
///
/// # Members
/// * `multi_threaded` - Boolean to indicate whether this executor is part of
/// multi threaded constellation or not. Used to indicate whether suspended
/// events should be pushed to parent or not.
/// * `work_queue` - Shared queue with Constellation instance, used to grab
/// work when available.
/// * `local_work` - Local queue with work, stolen jobs get put here before
/// executed, constellation can use this queue to load balance different
/// executors.
/// * `work_suspended` - Work which as been suspended (activity::State::suspend
/// was returned). This activity is triggered by sending receiving an event.
/// * `event_queue` - Shared queue for events containing data, executor will
/// check this queue whenever if is expecting events
/// * `event_suspended` - Events that have been received but have no activity
/// on this thread
/// * `constellation` - A reference to the InnerConstellation instance, required
/// by the functions in the activities executed
/// * `receiver` - Receiving channel used to get signals from parent
/// * `sender` - Sending channel used to signal parent
pub struct ExecutorThread {
    work_queue: Arc<Mutex<HashMap<ActivityIdentifier, Box<dyn ActivityWrapperTrait>>>>,
    work_suspended: Arc<Mutex<HashMap<ActivityIdentifier, Box<dyn ActivityWrapperTrait>>>>,
    event_queue: Arc<Mutex<EventQueue>>,
    constellation: Arc<Mutex<Box<dyn ConstellationTrait>>>,
    receiver: Receiver<bool>,
    sender: Sender<bool>,
    thread_id: i32,
}

impl ExecutorThread {
    /// Create a new ExecutorThread
    ///
    /// # Arguments
    /// * `multi_threaded` - Boolean indicating whether this executor is part
    /// of multi threaded constellation or not. This is used to indicate
    /// whether to push suspended events/activities to the parent or locally.
    /// * `work_queue` - Injector queue of ActivityWrapperTraits which
    /// is shared with constellation instance
    /// * `event_queue` - Same as work_queue but for events
    /// * `constellation` - Shared constellation which can be used when
    /// processing activities
    ///
    /// # Returns
    /// * `ExecutorThread` - New executor thread which asynchronously processes
    /// events
    pub fn new(
        work_queue: Arc<Mutex<HashMap<ActivityIdentifier, Box<dyn ActivityWrapperTrait>>>>,
        work_suspended: Arc<Mutex<HashMap<ActivityIdentifier, Box<dyn ActivityWrapperTrait>>>>,
        event_queue: Arc<Mutex<EventQueue>>,
        constellation: Arc<Mutex<Box<dyn ConstellationTrait>>>,
        receiver: Receiver<bool>,
        sender: Sender<bool>,
        thread_id: i32,
    ) -> ExecutorThread {
        ExecutorThread {
            work_queue,
            work_suspended,
            event_queue,
            constellation,
            receiver,
            sender,
            thread_id
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
        let mut guard = self.work_queue.lock().unwrap();
        if guard.is_empty() {
            drop(guard);
            return None;
        }

        let mut key= None;
        let mut activity: Option<Box<dyn ActivityWrapperTrait>> = None;

        let mut it = guard.keys().take(1).map(|x|
            key = Some(x.clone())
        );
        it.next();

        if key.is_some() {
            activity = guard.remove(&key.unwrap());
        }
        drop(guard);

        activity
    }

    /// Executes a stolen activity. It starts with the initialize(..) function,
    /// continues with process(..) and ends with cleanup.
    ///
    /// If an activity function returns activity::State::suspend, it will add
    /// the activity to the "work_suspended" queue and return. This activity
    /// can then be re-activated by receiving an event.
    ///
    /// When an activity is re-activated with an event, it will start with
    /// the *process* function
    ///
    /// # Arguments
    /// * `activity` - A boxed activity to perform work on.
    fn run_activity(&mut self, mut activity: Box<dyn ActivityWrapperTrait>) {
        let aid = activity.activity_identifier().clone();

        // Initialize
        match activity.initialize(self.constellation.clone(), &aid) {
            activity::State::SUSPEND => {
                // Activity must suspend, add to suspended queue and
                // stop processing

                self.work_suspended.lock().unwrap().insert(aid, activity);
                return;
            }
            activity::State::FINISH => {}
        }

        // TODO, move this to the top
        let mut event: Option<Box<Event>> = None;

        if activity.expects_event() {
            event = self.event_queue.lock().unwrap().remove(aid.clone());
            if event.is_none() {
                self.work_suspended.lock().unwrap().insert(aid, activity);
                return;
            }
        }

        self.process(activity, event);
    }

    /// Start the process function on an activity and handle return value
    /// appropriately (can be suspend or finish). Upon finish, the cleanup
    /// function will be called on the activity.
    fn process(&mut self, mut activity: Box<dyn ActivityWrapperTrait>, e: Option<Box<Event>>) {
        let aid = activity.activity_identifier().clone();

        match activity.process(self.constellation.clone(), e, &aid) {
            activity::State::SUSPEND => {
                // Activity must suspend, add to suspended queue and
                // stop processing
                self.work_suspended.lock().unwrap().insert(aid, activity);
                return;
            }
            activity::State::FINISH => {
                // Cleanup activity
                activity.cleanup(self.constellation.clone());
            }
        }
    }

    /// Returns whether there is something left in the queues
    ///
    /// # Returns
    /// * `bool` - Boolean
    ///     - true: There are remaining items
    ///     - false: THere are no remaining items
    pub fn queues_empty(&self) -> bool {
        if self.work_queue.lock().unwrap().is_empty()
            && self.work_suspended.lock().unwrap().is_empty()
            && self.event_queue.lock().unwrap().is_empty()
        {
            return true;
        }

        false
    }

    fn check_suspended_work(&mut self) {
        let keys: Vec<ActivityIdentifier> = self.work_suspended.lock().unwrap().keys().map(|x| x.clone()).collect();
        for key in keys {
            let event = self.event_queue.lock().unwrap().remove(key.clone());

            if event.is_some() {
                // We have received the event!
                let activity = self.work_suspended.lock().unwrap().remove(&key);
                if activity.is_some() {
                    self.process(activity.unwrap(), event);
                } else {
                    // For thread safety
                    self.event_queue.lock().unwrap().insert(key, event.unwrap());
                }
            }
        }
    }

    /// This will startup the thread, periodically check for work forever or
    /// if shut down from parent Constellation.
    pub fn run(&mut self) {
        // Wait for signal for 10 microseconds before proceeding
        let time = time::Duration::from_micros(SLEEP_TIME);

        loop {
            // Check if we have received event for work
            if !self.work_suspended.lock().unwrap().is_empty() {
                // Process event
                self.check_suspended_work();
            }

            // Check for fresh work
            match self.check_for_work() {
                Some(x) => self.run_activity(x),
                None => (),
            }

            // Check for signal to shut down
            if let Ok(val) = self.receiver.recv_timeout(time) {
                if val {
                    info!("Got signal to shutdown");

                    if self.queues_empty() {
                        // Signal that we are shutting down
                        self.sender.send(true).expect(
                            "Failed to send signal to \
                             InnerConstellation from executor thread",
                        );
                        return; // Shutdown thread
                    } else {
                        self.sender.send(false).expect(
                            "Failed to send signal to \
                             InnerConstellation from executor thread",
                        );
                    }
                }
            }
        }
    }
}