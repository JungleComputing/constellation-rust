use super::activity::ActivityTrait;
use super::constellation_identifier::ConstellationIdentifier;
use super::context::Context;
use super::event::Event;
use super::implementation::error::ConstellationError;
use crate::activity_identifier::ActivityIdentifier;
use std::sync::{Arc, Mutex};

/// Main trait for Constellation, use for setting up a Constellation instance,
/// specifying properties and configurations.
pub trait ConstellationTrait: Sync + Send + mopa::Any {
    /// Activate Constellation instance.
    ///
    /// When created, the Constellation instance is inactive in order for the
    /// user to be able to change configuration and properties before
    /// activation.
    ///
    /// # Returns
    /// * `Result<bool, ConstellationError` - Result<..> containing possible
    /// error information.
    fn activate(&mut self) -> Result<bool, ConstellationError>;

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
    /// * `src` - the boxed activity sending the event
    /// * `e` - the event to send
    fn send(&mut self, e: Box<Event>);

    /// Terminate Constellation instance.
    ///
    /// # Returns
    /// * `Result<bool, ConstellationError` - Result<..> containing possible
    ///  error information.
    fn done(&mut self) -> Result<bool, ConstellationError>;

    /// Return the identifier for this Constellation instance
    ///
    /// # Returns
    /// * `ConstellationIdentifier` - An identifier for this specific
    /// Constellation instance
    fn identifier(&mut self) -> ConstellationIdentifier;

    /// Check if the calling node is master.
    ///
    /// # Returns
    /// * `Result<bool, ConstellationError` - Result<..>, which upon a
    /// successful call contains *true* if node is master and *false* if not.
    fn is_master(&mut self) -> Result<bool, ConstellationError>;

    fn nodes(&mut self) -> i32;

    fn set_parent(&mut self, parent: Arc<Mutex<Box<dyn ConstellationTrait>>>);
}

mopafy!(ConstellationTrait);
