use crate::activity;
use crate::activity::ActivityTrait;
use crate::constellation::ConstellationTrait;
use crate::event::Event;
use std::sync::{Arc, Mutex};

pub struct SingleEventCollector {
    event: Option<Event>,
}

impl ActivityTrait for SingleEventCollector {
    fn cleanup(&mut self, _: &ConstellationTrait) {
        // no cleanup necessary
    }

    fn initialize(&mut self, _: &ConstellationTrait) -> usize {
        // Don't process anything, just suspend for later processing
        return activity::SUSPEND;
    }

    fn process(&mut self, _: &ConstellationTrait, event: Event) -> usize {
        // TODO Notify wait_for_event that event has been received
        self.event = Some(event);

        return activity::FINISH;
    }
}

impl SingleEventCollector {
    pub fn new() -> Arc<Mutex<dyn ActivityTrait>> {
        Arc::from(Mutex::from(SingleEventCollector {
            event: None,
        }))
    }

    /// This method blocks waiting until an event has been received, upon which
    /// it returns the event
    ///
    /// # Returns
    /// * `Event` -> The event received
    pub fn wait_for_event(&self) -> Event {
        // TODO This method needs to be implemented
        unimplemented!();
    }
}
