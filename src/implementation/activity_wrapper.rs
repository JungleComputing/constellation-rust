use super::super::activity::ActivityTrait;
use super::super::activity_identifier::ActivityIdentifier;
use super::constellation_identifier::ConstellationIdentifier;
use std::sync::{Mutex, Arc};
use crate::context::Context;
use crate::constellation::ConstellationTrait;
use crate::event::Event;

pub trait ActivityWrapperTrait: Sync + Send {
    fn identifier(&self) -> ActivityIdentifier;
}

/// Structure for internal use inside Constellation only
pub struct ActivityWrapper {
    id: ActivityIdentifier,
    may_be_stolen: bool,
    context: Context,
    expects_events: bool,
    activity: Arc<Mutex<dyn ActivityTrait>>,
}

impl ActivityWrapperTrait for ActivityWrapper {
    fn identifier(&self) -> ActivityIdentifier {
        self.id.clone()
    }
}

impl ActivityTrait for ActivityWrapper {
    fn cleanup(&mut self, constellation: &ConstellationTrait) {
        self.activity.lock().expect(
            &format!("Could not acquire lock on activity with id {}", self.identifier())
        ).cleanup(constellation);
    }

    fn initialize(&mut self, constellation: &ConstellationTrait) -> usize {
        self.activity.lock().expect(
            &format!("Could not acquire lock on activity with id {}", self.identifier())
        ).initialize(constellation)
    }

    fn process(&mut self, constellation: &ConstellationTrait, event: Event) -> usize{
        self.activity.lock().expect(
            &format!("Could not acquire lock on activity with id {}", self.identifier())
        ).process(constellation, event)
    }
}

impl ActivityWrapper {
    pub fn new(
        const_id: &ConstellationIdentifier,
        activity: &Arc<Mutex<dyn ActivityTrait>>,
        context: &Context,
        may_be_stolen: bool,
        expects_events: bool,
    ) -> Box<ActivityWrapper> {

        Box::from(ActivityWrapper {
            id: ActivityIdentifier::new(const_id),
            context: (*context).clone(),
            may_be_stolen,
            expects_events,
            activity: activity.clone(), // Clone the reference
        })
    }
}
