///! Main module for Constellation, use for setting up a Constellation instance,
///! specifying properties and configurations. See SingleThreadedConstellation
///! and MultiThreadedConstellation for examples.


use crate::error::ConstellationError;
use crate::{ActivityTrait, Context, Event, ActivityIdentifier};
use crate::implementation::constellation_identifier::ConstellationIdentifier;

use std::sync::{Arc, Mutex};


/// Has to implement Sync and Send to be able to be shared in Arc<Mutex<..>>
/// between threads. mopa::Any enables downcasting on the trait object.
pub trait ConstellationTrait: Sync + Send + mopa::Any {
    /// Activate Constellation instance.
    ///
    /// When created, the Constellation instance is inactive in order for the
    /// user to be able to change configuration and properties before
    /// activation.
    ///
    /// # Returns
    /// * `Result<bool, ConstellationError` - Result struct which contains a
    /// boolean to indicate whether activation was successful or not. Upon
    /// error it will return ConstellationError
    fn activate(&mut self) -> Result<bool, ConstellationError>;

    /// Submit an activity to Constellation. Make sure to handle the
    /// ActivityIdentifier properly so that it is unique for the entire
    /// constellation instance.
    ///
    /// # Arguments
    /// * `activity` - A reference to an activity implementing the ActivityTrait.
    /// The activity must be inside an Arc<Mutex<..>>, in order to work with
    /// thread safety.
    /// * `context` - A reference to the context created for this activity
    /// * `may_be_stolen` - A boolean indicating whether this activity can be
    /// stolen or not.
    /// * `expects_events` - A boolean indicating whether this activity expects
    /// events or not.
    ///
    /// # Returns
    /// * `ActivityIdentifier` - The generated Activity Identifier for
    /// this Activity.
    fn submit(
        &mut self,
        activity: Arc<Mutex<dyn ActivityTrait>>,
        context: &Context,
        may_be_stolen: bool,
        expects_events: bool,
    ) -> ActivityIdentifier;

    /// Send an event
    ///
    /// # Arguments
    /// * `e` - The event to send, an event may contain a user-defined Payload
    /// struct, containing data.
    fn send(&mut self, e: Box<Event>);

    /// Terminate Constellation instance.
    ///
    /// # Returns
    /// * `Result<bool, ConstellationError` - Result which contains a boolean
    /// indicating whether Constellation successfully shutdown, upon error
    /// a ConstellationError will be returned.
    fn done(&mut self) -> Result<bool, ConstellationError>;

    /// Return the identifier for this Constellation instance
    ///
    /// # Returns
    /// * `ConstellationIdentifier` - An identifier for this specific
    /// Constellation instance.
    fn identifier(&mut self) -> ConstellationIdentifier;

    /// Check if the calling node is master.
    ///
    /// # Returns
    /// * `Result<bool, ConstellationError` - Result<..>, which upon a
    /// successful call contains *true* if node is master and *false* if not.
    fn is_master(&self) -> Result<bool, ConstellationError>;

    /// Return the number of nodes in this constellation instance.
    fn nodes(&mut self) -> i32;
}

mopafy!(ConstellationTrait);
