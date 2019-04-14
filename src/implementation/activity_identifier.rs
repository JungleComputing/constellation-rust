use crate::implementation::communication::node_handler::NodeHandler;
use crate::implementation::constellation_identifier::ConstellationIdentifier;

use std::fmt;
use std::sync::Arc;
use std::sync::Mutex;

/// Struct that holds an unique identification about a newly submitted activity.
/// The ActivityIdentifier is automatically generated when creating a
/// ActivityWrapper using the new(..) method, in order to assure that it is
/// unique across in the entire constellation instance. NEVER generate an
/// ActivityIdentifier separately, as it might get the same ID as some other
/// identifier and cause faulty executions.
///
/// Two ActivityIdentifiers can be compared to each other and displayed on
/// the screen.
///
/// # Members
/// * `constellation_id` - Constellation identifier
/// * `node_info` - Information about the node, such as node id, node name,
/// and number of threads.
/// * `activity_id` - A u64 number, unique for this ActivityIdentifier
#[derive(Debug, Clone, Hash)]
pub struct ActivityIdentifier {
    pub constellation_id: i32,
    pub node_info: NodeHandler,
    pub activity_id: u64,
}

impl fmt::Display for ActivityIdentifier {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "CID:{}:NID:{}:AID:{}",
            self.constellation_id, self.node_info.node_id, self.activity_id
        )
    }
}

impl ActivityIdentifier {
    /// Generate a new ActivityIdentifier, this method should only be called
    /// internally from the ActivityWrapper. NEVER generate an
    /// ActivityIdentifier in the application.
    ///
    /// # Arguments
    /// * `const_id` - Arc reference to the constellation identifier.
    ///
    /// # Returns
    /// * `ActivityIdentifier` - Unique ActivityIdentifier for this
    /// constellation instance.
    pub fn new(const_id: Arc<Mutex<ConstellationIdentifier>>) -> ActivityIdentifier {
        let mut const_id = const_id.lock().unwrap();

        ActivityIdentifier {
            constellation_id: const_id.constellation_id,
            node_info: NodeHandler {
                node_name: const_id.node_info.node_name.clone(),
                node_id: const_id.node_info.node_id,
            },
            activity_id: const_id.generate_activity_id(),
        }
    }
}

impl PartialEq for ActivityIdentifier {
    fn eq(&self, other: &ActivityIdentifier) -> bool {
        (self.activity_id == other.activity_id)
            && (self.node_info.node_id == other.node_info.node_id)
            && (self.constellation_id == other.constellation_id)
    }
}

impl Eq for ActivityIdentifier {}
