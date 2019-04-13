use crate::event::Event;
use crate::activity::ActivityTrait;
use std::sync::{Arc, Mutex};
use crate::context::Context;
use crate::activity_identifier::ActivityIdentifier;
use crate::implementation::activity_wrapper::{ActivityWrapperTrait, ActivityWrapper};
use crate::constellation::ConstellationTrait;

use hashbrown::HashMap;
use crossbeam::{Sender, Receiver, deque, deque::Steal};
use std::time;
use crate::constellation_identifier::ConstellationIdentifier;
use super::mpi::environment::Universe;
use crate::implementation::error::ConstellationError;
use crate::implementation::event_queue::EventQueue;

const TIME_BETWEEN_CHECKS: u64 = 1000;

#[derive (Clone)]
pub struct ExecutorQueues {
    pub const_id: Arc<Mutex<ConstellationIdentifier>>,
    pub activities: Arc<Mutex<HashMap<ActivityIdentifier, Box<dyn ActivityWrapperTrait>>>>,
    pub activities_suspended: Arc<Mutex<HashMap<ActivityIdentifier, Box<dyn ActivityWrapperTrait>>>>,
    pub event_queue: Arc<Mutex<EventQueue>>,
}

impl ExecutorQueues {
    pub fn new(universe: &Universe, activity_counter: Arc<Mutex<u64>>, thread_id: i32) -> ExecutorQueues {
        ExecutorQueues {
            const_id: Arc::new(Mutex::new(ConstellationIdentifier::new(&universe, activity_counter, thread_id))),
            activities: Arc::new(Mutex::new(HashMap::new())),
            activities_suspended: Arc::new(Mutex::new(HashMap::new())),
            event_queue: Arc::new(Mutex::new(EventQueue::new())),
        }
    }

    pub fn new_existing(const_id: Arc<Mutex<ConstellationIdentifier>>,
                        work_queue: Arc<Mutex<HashMap<ActivityIdentifier, Box<dyn ActivityWrapperTrait>>>>,
                        work_suspended: Arc<Mutex<HashMap<ActivityIdentifier, Box<dyn ActivityWrapperTrait>>>>,
                        event_queue: Arc<Mutex<EventQueue>>,
    ) -> ExecutorQueues {
        ExecutorQueues {
            const_id,
            activities: work_queue,
            activities_suspended: work_suspended,
            event_queue,
        }
    }
}

#[derive(Clone)]
pub struct ThreadHelper {
    activities: Arc<Mutex<deque::Injector<Box<dyn ActivityWrapperTrait>>>>,
    events: Arc<Mutex<deque::Injector<Box<Event>>>>,
}

impl ThreadHelper {
    pub fn new(activities: Arc<Mutex<deque::Injector<Box<dyn ActivityWrapperTrait>>>>,
               events: Arc<Mutex<deque::Injector<Box<Event>>>>,) -> ThreadHelper {
        ThreadHelper {
            activities,
            events,
        }
    }

    pub fn submit(&mut self, activity_wrapper: Box<ActivityWrapper>) {
        self.activities.lock().unwrap().push(activity_wrapper);
    }

    pub fn send(&mut self, e: Box<Event>) {
        self.events.lock().unwrap().push(e);
    }
}

#[derive(Clone)]
pub struct MultiThreadHelper {
    pub threads:  Vec<(Arc<Mutex<Box<dyn ConstellationTrait>>>, ExecutorQueues)>,
    time_between_checks: time::Duration,
    debug: bool,
    activities_from_threads: Arc<Mutex<deque::Injector<Box<dyn ActivityWrapperTrait>>>>,
    events_from_threads: Arc<Mutex<deque::Injector<Box<Event>>>>,
    local_events: Arc<Mutex<EventQueue>>,
}

impl MultiThreadHelper {
    pub fn new(debug: bool,
               activities_from_threads: Arc<Mutex<deque::Injector<Box<dyn ActivityWrapperTrait>>>>,
               events_from_threads: Arc<Mutex<deque::Injector<Box<Event>>>>,
    ) -> MultiThreadHelper {
        MultiThreadHelper {
            threads: Vec::new(),
            time_between_checks: time::Duration::from_micros(TIME_BETWEEN_CHECKS),
            debug,
            activities_from_threads,
            events_from_threads,
            local_events: Arc::new(Mutex::new(EventQueue::new())),
        }
    }

    pub fn new_from_vec(debug: bool,
                        threads:  Vec<(Arc<Mutex<Box<dyn ConstellationTrait>>>, ExecutorQueues)>,
                        activities_from_threads: Arc<Mutex<deque::Injector<Box<dyn ActivityWrapperTrait>>>>,
                        events_from_threads: Arc<Mutex<deque::Injector<Box<Event>>>>,
                        local_events: Arc<Mutex<EventQueue>>,
    ) -> MultiThreadHelper {
        MultiThreadHelper {
            threads,
            time_between_checks: time::Duration::from_micros(TIME_BETWEEN_CHECKS),
            debug,
            activities_from_threads,
            events_from_threads,
            local_events
        }
    }

    pub fn push(&mut self, executor_queues: ExecutorQueues, constellation: Arc<Mutex<Box<dyn ConstellationTrait>>>) {
        self.threads.push((constellation, executor_queues));
    }

    pub fn set_interval_time(&mut self, time: time::Duration) {
        self.time_between_checks = time;
    }

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

            // Check for signal to shut down
            if let Ok(val) = receiver.recv_timeout(self.time_between_checks) {
                if val {
                    // Signal that we are shutting down
                    sender.send(true).expect(
                        "Failed to send signal to \
                         InnerConstellation from executor thread",
                    );
                    return; // Shutdown thread
                }
            }
        }
    }

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

        let activity_wrapper = ActivityWrapper::new(
            const_id,
            activity,
            context,
            may_be_stolen,
            expects_events,
        );
        let aid = activity_wrapper.activity_identifier().clone();

        if self.debug {
            info!("Submitting activity with ID: {} to thread: {}", &aid, index);
        }

        self.threads[index].1.activities.lock().unwrap().insert(aid.clone(), activity_wrapper);

        aid
    }

    pub fn send(&mut self, e: Box<Event>) {
        if self.debug {
            info!("Send Event: {} -> {}", e.get_src(), e.get_dst());
        }
        self.distribute_event(e);
    }

    pub fn done(&mut self) -> Result<bool, ConstellationError> {
        for x in 0..self.threads.len() {
            if let Ok(res) = self.threads[x].0.lock().expect("Could not get lock on constellation instance").done(){
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

    fn get_thread_with_least_work(&mut self) -> usize {
        let mut shortest = u64::max_value();
        let mut index = 0;

        for i in 0..self.threads.len() {
            let length = self.threads[i].1.activities.lock().unwrap().len() + self.threads[i].1.activities_suspended.lock().unwrap().len();
            if length < shortest as usize {
                index = i;
                shortest = length as u64;
            }
        }

        index
    }

    fn distribute_event(&mut self, event: Box<Event>) {
        let key = event.get_dst();

        for i in 0..self.threads.len() {
            if self.threads[i].1.activities.lock().unwrap().contains_key(&key) ||
                self.threads[i].1.activities_suspended.lock().unwrap().contains_key(&key) {
                self.threads[i].1.event_queue.lock().unwrap().insert(key, event);
                return;
            }
        }


        if self.debug {
            info!(" to thread handler");
        }

        // Event does not exist in any activity yet, let it sit in our local
        // queue until we find a matching activity. This should in essence only
        // be possible when an event has an invalid destination, or is retrieved
        // from another node, without the matching activity
        self.local_events.lock().unwrap().insert(event.get_dst(), event);
    }

    fn handle_thread_events(&mut self) {
        loop {
            let event = self.events_from_threads.lock().unwrap().steal();
            match event {
                Steal::Success(e) => {
                    self.distribute_event(e);
                },
                _ => {
                    return;
                }
            }
        }
    }

    fn distribute_activity(&mut self, activity_trait: Box<dyn ActivityWrapperTrait>) {
        let index = self.get_thread_with_least_work();

        let aid = activity_trait.activity_identifier();

        self.threads[index].1.activities.lock().unwrap().insert(aid.clone(), activity_trait);
    }

    fn handle_local_events(&mut self) {
        let mut guard = self.local_events.lock().unwrap();
        if guard.is_empty() {
            drop(guard);
            return;
        }

        let mut key = None;
        let mut event: Option<Box<Event>> = None;

        let mut it = guard.keys().take(1).map(|x|
            key = Some(x.clone())
        );
        it.next();

        if key.is_some() {
            event = guard.remove(key.unwrap());
        }
        drop(guard);
    }

    fn handle_thread_activity(&mut self) {
        loop {
            // Load balance activites
            let activity = self.activities_from_threads.lock().unwrap().steal();
            match activity {
                Steal::Success(activity) => {
                    self.distribute_activity(activity);
                },
                _ => {
                    return;
                }
            }

            // Make sure event goes to correct thread
            let event = self.events_from_threads.lock().unwrap().steal();
            match event {
                Steal::Success(e) => {
                    self.distribute_event(e);
                },
                _ => {
                    return;
                }
            }

            // Check local events
            self.handle_local_events();
        }
    }
}