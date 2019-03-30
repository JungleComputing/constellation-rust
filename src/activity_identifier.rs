use super::constellation_identifier::ConstellationIdentifier;
use super::implementation::communication::node_handler::NodeHandler;
use std::fmt;
use std::sync::Arc;
use std::sync::Mutex;

pub trait ActivityIdentifierTrait {
    fn to_string(&self) -> String;
}

#[derive(Debug, Clone)]
pub struct ActivityIdentifier {
    pub constellation_id: i32,
    pub node_info: NodeHandler,
    pub activity_id: u64,
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
    pub fn new(const_id_arc: Arc<Mutex<ConstellationIdentifier>>) -> ActivityIdentifier {
        let mut const_id = const_id_arc.lock().unwrap();

        ActivityIdentifier {
            constellation_id: const_id.constellation_id,
            node_info: NodeHandler{
                node_name: const_id.node_info.node_name.clone(),
                node_id: const_id.node_info.node_id
            },
            activity_id: const_id.generate_activity_id(),
        }
    }
}
