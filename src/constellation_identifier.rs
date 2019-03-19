use super::implementation::communication::node_handler;
use mpi::environment::Universe;
use mpi::topology::Communicator;
use mpi::topology::Rank;
use std::collections::HashMap;
use std::fmt;

#[derive(Debug)]
pub struct ConstellationIdentifier {
    pub constellation_id: i32,
    pub node_info: node_handler::NodeHandler,
    pub group: HashMap<Rank, node_handler::NodeHandler>, // All processes and their node information
}

impl ConstellationIdentifier {
    pub fn new(universe: &Universe) -> ConstellationIdentifier {
        let world = universe.world();
        let rank = world.rank();

        let mut const_id = ConstellationIdentifier {
            constellation_id: 0,
            node_info: node_handler::NodeHandler {
                node_name: mpi::environment::processor_name().expect(
                    "Could not retrieve processor_name"
                ),
                node_id: 0
            },
            group: HashMap::new(),
        };

        // Create mpi groups to track processes on each node
        node_handler::create_groups(&mut const_id.group, &universe);

        const_id.node_info.node_id = const_id.group.get(&rank).unwrap().node_id;

        const_id
    }

    pub fn new_empty() -> ConstellationIdentifier {
        ConstellationIdentifier {
            constellation_id: 0,
            node_info: node_handler::NodeHandler {
                node_name: "EMPTY".to_string(),
                node_id: 0
            },
            group: HashMap::new(),
        }
    }

    pub fn to_string(&self) -> String {
        String::from(format!("CID:{}:{}", self.constellation_id, self.node_info.node_id))
    }
}

impl fmt::Display for ConstellationIdentifier {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "CID:{}:{}", self.constellation_id, self.node_info.node_id)
    }
}

impl Clone for ConstellationIdentifier {
    fn clone(&self) -> Self {
        ConstellationIdentifier {
            constellation_id: self.constellation_id.clone(),
            node_info: self.node_info.clone(),
            group: HashMap::new(),
        }
    }
}
