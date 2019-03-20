use super::message::MessageTrait;

pub struct Event {
    message: Box<dyn MessageTrait>,
}

impl Event {
    pub fn new(message: Box<dyn MessageTrait>) -> Event {
        Event {
            message,
        }
    }

    fn to_string(&self) -> &String {
        &self.message.to_string()
    }

    pub fn get_message(&self) -> &Box<dyn MessageTrait> {
        &self.message
    }
}
