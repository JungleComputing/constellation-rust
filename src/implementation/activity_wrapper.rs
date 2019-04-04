use std::fmt;
use std::sync::{Arc, Mutex};

use crate::activity::{ActivityTrait, State};
use crate::activity_identifier::ActivityIdentifier;
use crate::constellation::ConstellationTrait;
use crate::constellation_identifier::ConstellationIdentifier;
use crate::context::Context;
use crate::event::Event;

pub trait ActivityWrapperTrait: Sync + Send + ActivityTrait + fmt::Display {
    fn activity_identifier(&self) -> &ActivityIdentifier;
    fn expects_event(&self) -> bool;
    fn may_be_stolen(&self) -> bool;
}

/// Structure for internal use inside Constellation only. As soon as an
/// activity is created, it will be wrapped by an ActivityWrapper.
///
/// The wrapper contains useful information such as an ActivityIdentifier and
/// whether the activity may be stolen. The logic
/// (and to some extent this information), is hidden to the user.
///
/// # Members
/// * `id` - A generated activity identifier, unique for this activity
/// * `may_be_stolen` - Indicates whether this activity may be stolen by other
/// threads/nodes
/// * `context` - The context specifying where an activity may be executed
/// * `expects_events` - Indicates whether this activity expects events to
/// complete
/// * `activity` - A user defined activity to be executed in Constellation
/// * `constellation_id` - A reference to an identifier, identifying this
/// constellation instance.
pub struct ActivityWrapper {
    id: ActivityIdentifier,
    may_be_stolen: bool,
    context: Context,
    expects_events: bool,
    activity: Arc<Mutex<dyn ActivityTrait>>,
    constellation_id: Arc<Mutex<ConstellationIdentifier>>,
}

impl ActivityWrapperTrait for ActivityWrapper {
    fn activity_identifier(&self) -> &ActivityIdentifier {
        &self.id
    }

    fn expects_event(&self) -> bool {
        return self.expects_events
    }

    fn may_be_stolen(&self) -> bool {
        return self.may_be_stolen
    }
}

impl ActivityTrait for ActivityWrapper {
    fn cleanup(&mut self, constellation: Arc<Mutex<Box<dyn ConstellationTrait>>>) {
        self.activity
            .lock()
            .expect(&format!(
                "Could not acquire lock on activity with id {}",
                self.activity_identifier()
            ))
            .cleanup(constellation);
    }

    fn initialize(
        &mut self,
        constellation: Arc<Mutex<Box<dyn ConstellationTrait>>>,
        id: &ActivityIdentifier,
    ) -> State {
        assert_eq!(
            self.activity_identifier(),
            id,
            "Found different activity identifiers in wrapper \
             and argument: {} - {}",
            self.activity_identifier(),
            id
        );

        self.activity
            .lock()
            .expect(&format!(
                "Could not acquire lock on activity with id {}",
                id
            ))
            .initialize(constellation, id)
    }

    fn process(
        &mut self,
        constellation: Arc<Mutex<Box<dyn ConstellationTrait>>>,
        event: Option<Box<Event>>,
        id: &ActivityIdentifier,
    ) -> State {
        assert_eq!(
            self.activity_identifier(),
            id,
            "Found different activity identifiers in wrapper \
             and argument: {} - {}",
            self.activity_identifier(),
            id
        );

        self.activity
            .lock()
            .expect(&format!(
                "Could not acquire lock on activity with id {}",
                id
            ))
            .process(constellation, event, id)
    }
}

impl ActivityWrapper {
    pub fn new(
        const_id: Arc<Mutex<ConstellationIdentifier>>,
        activity: Arc<Mutex<dyn ActivityTrait>>,
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

impl fmt::Display for ActivityWrapper {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "{}:stealable:{}:{}:exp_event:{}",
            self.id, self.may_be_stolen, self.context, self.expects_events
        )
    }
}
