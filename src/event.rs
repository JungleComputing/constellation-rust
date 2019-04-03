use super::payload::PayloadTrait;
use crate::activity_identifier::ActivityIdentifier;
use std::fmt;

/// Event type, used for passing information between activities
///
/// * `src` - Holds the identifier of the src activity, this is set internally
/// upon submitting an event and is therefor None before sending.
/// * `dst` - Destination activity identifier
/// * `payload` - Data which should be communicated
#[derive(Clone, Debug)]
pub struct Event {
    src: ActivityIdentifier,
    dst: ActivityIdentifier,
    payload: Box<dyn PayloadTrait>,
}

impl Event {
    pub fn new(
        payload: Box<dyn PayloadTrait>,
        src: ActivityIdentifier,
        dst: ActivityIdentifier,
    ) -> Box<Event> {
        Box::new(Event { src, dst, payload })
    }

    pub fn get_payload(&self) -> &Box<dyn PayloadTrait> {
        &self.payload
    }

    pub fn get_src(&self) -> ActivityIdentifier {
        self.src.clone()
    }

    pub fn get_dst(&self) -> ActivityIdentifier {
        self.dst.clone()
    }
}

impl fmt::Display for Event {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "Src: {}\nDst: {}\nData: {:?}",
            self.src, self.dst, self.payload
        )
    }
}
