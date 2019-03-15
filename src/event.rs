pub struct Event {
    message: String,
}

impl Event {
    fn to_string(&self) -> String {
        String::from(format!("{}", self.message))
    }
}