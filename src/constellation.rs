/// Main trait for Constellation, use for setting up a Constellation instance,
/// specifying properties and configurations.
pub trait Constellation {
    /// Activate Constellation instance.
    ///
    /// When created, the Constellation instance is inactive in order for the
    /// user to be able to change configuration and properties before
    /// activation.
    ///
    /// # Returns
    /// * `Result<bool, ConstellationError` - Result<..> containing possible
    /// error information.
    fn activate() -> Result<bool, ConstellationError>;

    fn submit(activity: Activity);

    /// Send an event
    ///
    /// # Arguments
    /// * `Event` - the event to send
    fn send(e: Event);

    /// Terminate Constellation instance.
    ///
    /// # Returns
    /// * `Result<bool, ConstellationError` - Result<..> containing possible
    ///  error information.
    fn done() -> Result<bool, ConstellationError>;

    /// Return the identifier for this Constellation instance
    ///
    /// # Returns
    /// * `ConstellationIdentifier` - An identifier for this specific
    /// Constellation instance
    fn identifier() -> ConstellationIdentifier;

    /// Check if the calling node is master.
    ///
    /// # Returns
    /// * `Result<bool, ConstellationError` - Result<..>, which upon a
    /// successful call contains *true* if node is master and *false* if not.
    fn is_master() -> Result<bool, ConstellationError>;
}