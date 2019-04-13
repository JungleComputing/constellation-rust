use crate::activity;
use crate::activity::ActivityTrait;
use crate::activity_identifier::ActivityIdentifier;
use crate::constellation::ConstellationTrait;
use crate::event::Event;

use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;


/// Single event collector is an activity which only waits for one event. The
/// struct field event will be sent with this event upon receive.
///
/// # Members
/// * `event` - Event will be set when this activity retrieves the event.
pub struct SingleEventCollector {
    pub event: Option<Box<Event>>,
}

impl ActivityTrait for SingleEventCollector {
    fn cleanup(&mut self, _: Arc<Mutex<Box<dyn ConstellationTrait>>>) {
        // no cleanup necessary
    }

    fn initialize(
        &mut self,
        _: Arc<Mutex<Box<dyn ConstellationTrait>>>,
        _id: &ActivityIdentifier,
    ) -> activity::State {
        // Don't process anything, just suspend for later processing
        return activity::State::SUSPEND;
    }

    fn process(
        &mut self,
        _: Arc<Mutex<Box<dyn ConstellationTrait>>>,
        event: Option<Box<Event>>,
        _id: &ActivityIdentifier,
    ) -> activity::State {
        self.event = event;

        match &self.event {
            Some(_e) => {
                return activity::State::FINISH;
            }
            None => {
                return activity::State::SUSPEND;
            }
        }
    }
}

impl SingleEventCollector {
    pub fn new() -> Arc<Mutex<SingleEventCollector>> {
        Arc::from(Mutex::from(SingleEventCollector { event: None }))
    }

    /// Loop on the global field event and return it when it has a value
    ///
    /// # Arguments
    /// * `sec` - The SingleEventCollector to check event on.
    /// * `interval` - How often to check for the event
    pub fn get_event(sec: Arc<Mutex<SingleEventCollector>>, interval: Duration) -> Box<Event> {
        loop {
            let guard = sec.lock().unwrap();

            if let Some(event) = guard.event.clone() {
                return event;
            }

            // Release mutex
            drop(guard);
            thread::sleep(interval);
        }
    }
}
