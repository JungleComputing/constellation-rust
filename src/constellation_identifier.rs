pub struct ConstellationIdentifier {
    identifier: String
}

impl ConstellationIdentifier {
    pub fn get_id(self) -> String {
        self.identifier
    }

    pub fn new() -> ConstellationIdentifier {
        ConstellationIdentifier {
            // TODO This id should not be static, but be generated in a good
            // TODO unique way
            identifier: "MY_ID_1234".to_string()
        }
    }
}