use super::activity_identifier::ActivityIdentifier;
use super::constellation::ConstellationTrait;
use super::event::Event;

use std::sync::{Arc, Mutex};

/// State used to specify whether a method from an activity is done or requires
/// more data. The example below shows an initialization method which does not
/// require more data. The corresponding activity will immediately continue by
/// executing it's process method.
///
/// fn initialize(...) {
///     // Do initialization
///     return activity::State::FINISH;
/// }
pub enum State {
    FINISH,
    SUSPEND,
}

/// All activities must implement this trait and each function must return
/// a State (described above).
///
/// An activity may use all constellation methods provided in the
/// ConstellationTrait. They include, but are not limited to, submitting new
/// activities, processing data, sending data and notifying that the execution
/// is done.
///
/// See util/activities/ for various activities which may come in handy
/// See examples/ for some examples of what self-made activities may
/// look like
pub trait ActivityTrait: Sync + Send {
    fn cleanup(&mut self, constellation: Arc<Mutex<Box<dyn ConstellationTrait>>>);
    fn initialize(
        &mut self,
        constellation: Arc<Mutex<Box<dyn ConstellationTrait>>>,
        id: &ActivityIdentifier,
    ) -> State;
    fn process(
        &mut self,
        constellation: Arc<Mutex<Box<dyn ConstellationTrait>>>,
        event: Option<Box<Event>>,
        id: &ActivityIdentifier,
    ) -> State;
}
