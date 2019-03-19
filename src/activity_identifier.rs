use super::constellation_identifier::ConstellationIdentifier;
use super::implementation::communication::node_handler::NodeHandler;
use std::fmt;

pub trait ActivityIdentifierTrait {
    fn to_string(&self) -> String;
}

#[derive(Debug, Clone)]
pub struct ActivityIdentifier {
    constellation_id: i32,
    node_info: NodeHandler,
    activity_id: String,
}

impl ActivityIdentifierTrait for ActivityIdentifier {
    fn to_string(&self) -> String {
        format!(
            "{}:{}:{}",
            self.constellation_id, self.node_info.node_id, self.activity_id
        )
    }
}

impl fmt::Display for ActivityIdentifier {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "CID:{}:NID:{}:AID:{}", self.constellation_id, self.node_info.node_id, self.activity_id)
    }
}

impl ActivityIdentifier {
    pub fn new(const_id: &ConstellationIdentifier) -> ActivityIdentifier {
        ActivityIdentifier {
            constellation_id: const_id.constellation_id,
            node_info: NodeHandler{
                node_name: const_id.node_info.node_name.clone(),
                node_id: const_id.node_info.node_id
            },
            activity_id: "999".to_string(),
        }
    }
}
