pub struct Event {
    message: String,
}

impl Event {
    fn to_string(&self) -> String {
        String::from(format!("{}", self.message))
    }

    pub fn get_message(&self) -> String {
        self.to_string()
    }
}
