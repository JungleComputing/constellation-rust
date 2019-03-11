pub trait ActivityIdentifier {
    fn get_id(self) -> String;
}

struct ActivityIdentifierImpl {
    id: String,
}

impl ActivityIdentifier for ActivityIdentifierImpl {
    fn get_id(self) -> String {
        self.id
    }
}

impl ActivityIdentifierImpl {
    fn new() -> ActivityIdentifierImpl {
        ActivityIdentifierImpl {
            // TODO This id needs to be unique
            id: "activity-2".to_string()
        }
    }
}