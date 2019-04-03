extern crate crossbeam;

use std::sync::{Arc, Mutex};
use std::time;

use super::super::activity_wrapper::ActivityWrapperTrait;
use crate::activity;
use crate::activity_identifier::ActivityIdentifier;
use crate::constellation::ConstellationTrait;
use crate::event::Event;

use crossbeam::deque;
use crossbeam::deque::Steal;
use crossbeam::{Receiver, Sender};
use hashbrown::HashMap;

/// The executor thread runs in asynchronously and is in charge of executing
/// activities. It will periodically check for work/events in the Constellation
/// instance using it's shared queues. Closely coupled to inner_constellation.
///
/// When activities are re-activated with events after being suspended, they
/// will start by immediately calling the process method (possibly again).
///
/// # Members
/// * `work_queue` - Shared queue with Constellation instance, used to grab
/// work when available.
/// * `local_work` - Local queue with work, stolen jobs get put here before
/// executed, constellation can use this queue to load balance different
/// executors.
/// * `suspended_work` - Work which as been suspended (activity::State::suspend
/// was returned). This activity is triggered by sending receiving an event.
/// * `event_queue` - Shared queue for events containing data, executor will
/// check this queue whenever if is expecting events
/// * `events_waiting` - Events that have been received but have no activity
/// on this thread
/// * `constellation` - A reference to the InnerConstellation instance, required
/// by the functions in the activities executed
/// * `receiver` - Receiving channel used to get signals from parent
/// * `sender` - Sending channel used to signal parent
pub struct ExecutorThread {
    work_queue: Arc<Mutex<deque::Injector<Box<dyn ActivityWrapperTrait>>>>,
    local_work: deque::Worker<Box<dyn ActivityWrapperTrait>>,
    suspended_work: HashMap<ActivityIdentifier, Box<dyn ActivityWrapperTrait>>,
    event_queue: Arc<Mutex<deque::Injector<Box<Event>>>>,
    events_waiting: HashMap<ActivityIdentifier, Box<Event>>,
    constellation: Arc<Mutex<Box<dyn ConstellationTrait>>>,
    receiver: Receiver<bool>,
    sender: Sender<bool>,
}

impl ExecutorThread {
    /// Create a new ExecutorThread
    ///
    /// # Arguments
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
        work_queue: Arc<Mutex<deque::Injector<Box<dyn ActivityWrapperTrait>>>>,
        event_queue: Arc<Mutex<deque::Injector<Box<Event>>>>,
        constellation: Arc<Mutex<Box<dyn ConstellationTrait>>>,
        receiver: Receiver<bool>,
        sender: Sender<bool>,
    ) -> ExecutorThread {
        ExecutorThread {
            work_queue,
            local_work: deque::Worker::new_fifo(),
            suspended_work: HashMap::new(),
            event_queue,
            events_waiting: HashMap::new(),
            constellation,
            receiver,
            sender,
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
        self.local_work.pop().or_else(|| {
            if let Steal::Success(activity) = self
                .work_queue
                .lock()
                .unwrap()
                .steal_batch_and_pop(&self.local_work)
            {
                return Some(activity);
            } else {
                return None;
            }
        })
    }

    /// Executes a stolen activity. It starts with the initialize(..) function,
    /// continues with process(..) and ends with cleanup.
    ///
    /// If an activity function returns activity::State::suspend, it will add
    /// the activity to the "suspended_work" queue and return. This activity
    /// can then be re-activated by receiving an event.
    ///
    /// When an activity is re-activated with an event, it will start with
    /// the *process* function
    ///
    /// # Arguments
    /// * `activity` - A boxed activity to perform work on.
    fn run_activity(&mut self, mut activity: Box<dyn ActivityWrapperTrait>) {
        // Execute the initialize, process and cleanup methods of the stolen activity

        let aid = activity.activity_identifier().clone();

        // Initialize
        match activity.initialize(self.constellation.clone(), &aid) {
            activity::State::SUSPEND => {
                // Activity must suspend, add to suspended queue and
                // stop processing
                self.suspended_work.insert(aid, activity);
                return;
            }
            activity::State::FINISH => {}
        }

        // Check if we have an suspended event correlated to this activity
        let e = self.events_waiting.remove(&aid);

        self.process(activity, e);
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
                self.suspended_work.insert(aid, activity);
                return;
            }
            activity::State::FINISH => {
                // Cleanup activity
                activity.cleanup(self.constellation.clone());
            }
        }
    }

    /// Steal an event from "event_queue" which is shared with the
    /// SingleThreadedConstellation instance. If an event is stolen,
    /// it can proceed in two ways:
    ///     - The executor has a suspended activity waiting for the event,
    ///       the activity is re-activated with the event.
    ///     - There is no matching activity, add the event to the shared
    ///       "events_waiting" queue, it could be that the activity is somewhere
    ///       else, or that it has not yet arrived.
    fn steal_event(&mut self) -> bool {
        let data = self
            .event_queue
            .lock()
            .unwrap()
            .steal()
            .success()
            .expect("Error occurred when stealing an Event");
        let dst = data.get_dst();

        if let Some(activity) = self.suspended_work.remove(&dst) {
            assert_eq!(
                activity.activity_identifier(),
                &dst,
                "The destination ID of the event does not match the src ID \
                 of the suspended activity.\n{} - {}",
                dst,
                activity.activity_identifier()
            );

            self.process(activity, Some(data));
            return true;
        } else {
            // Key was not in suspended list,
            // store locally until activity is available
            self.events_waiting.insert(dst, data);
            return false;
        }
    }

    /// Go through all waiting events and check if there is a suspended activity
    /// waiting for any of them. If they match, the activity is immediately
    /// processed.
    fn find_activity_for_waiting_events(&mut self) {
        let mut to_process: Vec<ActivityIdentifier> = Vec::new();

        for key in self.events_waiting.keys() {
            if self.suspended_work.contains_key(key) {
                to_process.push(key.clone());
            }
        }

        for key in to_process {
            let activity = self.suspended_work.remove(&key).unwrap();

            assert_eq!(
                activity.activity_identifier(),
                &key,
                "The destination ID of the event does not match the src ID \
                 of the suspended activity.\n{} - {}",
                key,
                activity.activity_identifier()
            );

            let event = self.events_waiting.remove(&key);

            self.process(activity, event);
        }
    }

    /// This will startup the thread, periodically check for work forever or
    /// if shut down from InnerConstellation/SingleThreadedConstellation.
    pub fn run(&mut self) {
        // Wait for signal for 10 microseconds before proceeding
        let time = time::Duration::from_micros(10);

        loop {
            // Check for events from parent
            if !self.event_queue.lock().unwrap().is_empty() {
                // Process event
                self.steal_event();
                continue;
            }

            // Check queue of suspended events, if we have the matching activity
            if !self.events_waiting.is_empty() {
                self.find_activity_for_waiting_events();
                continue;
            }

            // Check for work
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
                            InnerConstellation from executor thread"
                        );
                        return;

                    } else {
                        self.sender.send(false).expect(
                            "Failed to send signal to \
                            InnerConstellation from executor thread"
                        );
                    }
                }
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
        if self.local_work.is_empty() &&
            self.suspended_work.is_empty() &&
            self.events_waiting.is_empty() {
            return true;
        }

        false
    }
}
