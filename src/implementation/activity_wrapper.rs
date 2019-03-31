use super::super::activity::ActivityTrait;
use super::super::activity_identifier::ActivityIdentifier;
use super::constellation_identifier::ConstellationIdentifier;
use std::sync::{Mutex, Arc};
use crate::context::Context;
use crate::constellation::ConstellationTrait;
use crate::event::Event;

pub trait ActivityWrapperTrait: Sync + Send + ActivityTrait {
    fn activity_identifier(&self) -> ActivityIdentifier;
}

/// Structure for internal use inside Constellation only
pub struct ActivityWrapper {
    id: ActivityIdentifier,
    may_be_stolen: bool,
    context: Context,
    expects_events: bool,
    activity: Arc<Mutex<dyn ActivityTrait>>,
    constellation_id: Arc<Mutex<ConstellationIdentifier>>
}

impl ActivityWrapperTrait for ActivityWrapper {
    fn activity_identifier(&self) -> ActivityIdentifier {
        self.id.clone()
    }
}

impl ActivityTrait for ActivityWrapper {
    fn cleanup(&mut self,  constellation: Arc<Mutex<Box<dyn ConstellationTrait>>>) {
        self.activity.lock().expect(
            &format!("Could not acquire lock on activity with id {}", self.activity_identifier())
        ).cleanup(constellation);
    }

    fn initialize(&mut self,  constellation: Arc<Mutex<Box<dyn ConstellationTrait>>>) -> usize {
        self.activity.lock().expect(
            &format!("Could not acquire lock on activity with id {}", self.activity_identifier())
        ).initialize(constellation)
    }

    fn process(&mut self,  constellation: Arc<Mutex<Box<dyn ConstellationTrait>>>, event: Event) -> usize{
        self.activity.lock().expect(
            &format!("Could not acquire lock on activity with id {}", self.activity_identifier())
        ).process(constellation, event)
    }
}

impl ActivityWrapper {
    pub fn new(
        const_id: Arc<Mutex<ConstellationIdentifier>>,
        activity: &Arc<Mutex<dyn ActivityTrait>>,
        context: &Context,
        may_be_stolen: bool,
        expects_events: bool,
    ) -> Box<ActivityWrapper> {

        // Create a new reference to ConstellationIdentifier
        let new_const_id = const_id.clone();

        Box::from(ActivityWrapper {
            id: ActivityIdentifier::new(const_id),
            context: (*context).clone(),
            may_be_stolen,
            expects_events,
            activity: activity.clone(), // Clone the reference
            constellation_id: new_const_id,
        })
    }
}