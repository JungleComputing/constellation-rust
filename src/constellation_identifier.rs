use mpi::environment::Universe;
use mpi::topology::Communicator;
use std::fmt;

#[derive(Debug)]
pub struct ConstellationIdentifier {
    pub constellation_id: i32,
    pub node_id: String,
    pub thread_id: i32,
}

impl ConstellationIdentifier {
    pub fn new(universe: &Universe) -> ConstellationIdentifier {
        let world = universe.world();
        let rank = world.rank();

        ConstellationIdentifier {
            constellation_id: 0,
            node_id: "EMPTY".to_string(),
            thread_id: rank,
        }
    }

    pub fn new_empty() -> ConstellationIdentifier {
        ConstellationIdentifier {
            constellation_id: 0,
            node_id: String::from("Empty"),
            thread_id: 0,
        }
    }

    pub fn to_string(&self) -> String {
        String::from(format!("CID:{}:{}", self.constellation_id, self.node_id))
    }
}

impl fmt::Display for ConstellationIdentifier {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "CID:{}:{}", self.constellation_id, self.node_id)
    }
}

impl Clone for ConstellationIdentifier {
    fn clone(&self) -> Self {
        ConstellationIdentifier {
            constellation_id: self.constellation_id.clone(),
            node_id: self.node_id.clone(),
            thread_id: self.thread_id.clone(),
        }
    }
}
