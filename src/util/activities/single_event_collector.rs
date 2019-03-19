use crate::activity;
use crate::activity::ActivityTrait;
use crate::constellation::ConstellationTrait;
use crate::event::Event;
use std::sync::{Arc, Mutex};

pub struct SingleEventCollector {}

impl ActivityTrait for SingleEventCollector {
    fn cleanup(&self, _: &ConstellationTrait) {
        // no cleanup necessary
    }

    fn initialize(&self, _: &ConstellationTrait) -> usize {
        // Don't process anything, just suspend for later processing
        return activity::SUSPEND;
    }

    fn process(&self, _: &ConstellationTrait, event: Event) -> usize {
        // Print hello world upon execution

        println!("{}", event.get_message());

        return activity::FINISH;
    }
}

impl SingleEventCollector {
    pub fn new() -> Arc<Mutex<dyn ActivityTrait>> {
        Arc::from(Mutex::from(SingleEventCollector {}))
    }
}
