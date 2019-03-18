pub trait ActivityIdentifier {
    fn to_string(&self) -> String;
}

pub struct ActivityIdentifierImpl {
    constellation_id: String,
    node_id: String,
    activity_id: String,
}

impl ActivityIdentifier for ActivityIdentifierImpl {
    fn to_string(&self) -> String {
        format!(
            "{}:{}:{}",
            self.constellation_id, self.node_id, self.activity_id
        )
    }
}

impl ActivityIdentifierImpl {
    fn new() -> ActivityIdentifierImpl {
        ActivityIdentifierImpl {
            constellation_id: "CID".to_string(),
            node_id: "NID".to_string(),
            activity_id: "AID".to_string(),
        }
    }
}
