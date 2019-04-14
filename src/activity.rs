///! An activity is where computation may take place, when executing an
///! application in constellation. Activities submitted to Constellation must
///! extend the ActivityTrait. That way Constellation can distribute and process
///! them correctly.
///!
///! When creating an activity, the `process(..)` method should contain the
///! heavy execution. The `initialize(..)` and `cleanup(..)` should only
///! initialize and cleanup necessary data structures related to the activity.
///!
///! IMPORTANT: Both the `initialize(..)` and `cleanup(..)` methods will only
///! be run ONCE, throughout the lifetime of an activity. If an activity is
///! suspended while waiting for an Event containing data, the process method
///! will always be executed upon receiving this event.
///!
///! See util/activities/ for various pre-made activities which may come in handy
///! See examples/ for some examples of what self-made activities may
///! look like.
use super::activity_identifier::ActivityIdentifier;
use super::constellation::ConstellationTrait;
use super::event::Event;

use std::sync::{Arc, Mutex};

/// State used to specify whether a method from an activity is done or requires
/// more data.
///
/// The example below shows a possible initialization method of an activity.
///
/// # Example
///
/// ```
/// fn initialize(...) -> State::FINISH {
///     let expects_activity = true;
///
///     if expects_activity {
///         // Expects an activity and should suspend until that activity is
///         // received
///         return State::Suspend;
///     }
///
///     // Continue immediately with the process(..) method
///     return State::FINISH;
/// }
/// ```
pub enum State {
    FINISH,
    SUSPEND,
}

/// All activities must implement this trait and each function must return
/// a State (described above).
///
/// An activity may use all constellation methods provided in the
/// ConstellationTrait, by using the `constellation` argument.
/// They include, but are not limited to, submitting new
/// activities, processing data, sending data and notifying that the execution
/// is done.
pub trait ActivityTrait: Sync + Send + mopa::Any {
    /// This method is called after the process method has returned FINISH,
    /// after this method returns the activity will be destroyed.
    ///
    /// # Arguments
    /// * `constellation` - The constellation instance upon which to call
    /// methods such as submit and send for newly created activities and events
    fn cleanup(&mut self, constellation: Arc<Mutex<Box<dyn ConstellationTrait>>>);

    /// Called immediately when a activity gets "activated" by an
    /// `ExecutorThread`.
    ///
    /// IMPORTANT!! THis method is always just called ONCE! An activity always
    /// starts with the process(..) method after being suspended
    ///
    /// # Arguments
    /// * `constellation` - Reference to constellation instance, used to submit
    /// new activities and events
    /// * `id` - ID for this activity, created internally
    fn initialize(
        &mut self,
        constellation: Arc<Mutex<Box<dyn ConstellationTrait>>>,
        id: &ActivityIdentifier,
    ) -> State;

    /// This method is called whenever a activity is reactivated after being
    /// suspended, or if it was just picked up by a `ExecutorThread` and
    /// returned State::FINISH from the `initialize(..)`
    ///
    /// # Arguments
    /// * `constellation` - Reference to the constellation instance used to
    /// submit new activities and events
    /// * `event` - Event containing `Payload` which can be processed by the
    /// activity. The event is of type Option<..> and will have the value None
    /// in case no event was passed (for example if called right after the
    /// initialize method completes).
    ///
    /// # Returns
    /// * `State` - The state of which to put the activity after finishing the
    /// process method.
    ///     - SUSPENDED: Activity is not yet done, add to suspended work and
    ///       wait for an Event to trigger activation.
    ///     - FINISH: In case the activity is done and should exit, the
    ///       `cleanup(..)` method will be called next
    fn process(
        &mut self,
        constellation: Arc<Mutex<Box<dyn ConstellationTrait>>>,
        event: Option<Box<Event>>,
        id: &ActivityIdentifier,
    ) -> State;
}

mopafy!(ActivityTrait);
