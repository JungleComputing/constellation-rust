use mpi::environment::Universe;
use mpi::topology::Communicator;

#[derive(Debug, Clone)]
pub struct ConstellationIdentifier {
    constellation_id: String,
    node_id: String,
}

impl ConstellationIdentifier {
    pub fn new(universe: &Universe) -> ConstellationIdentifier {

        let world = universe.world();
        let rank = world.rank();

        ConstellationIdentifier {
            constellation_id: "asdf".to_string(),
            node_id: "asdf".to_string(),
        }
    }

    pub fn new_empty() -> ConstellationIdentifier {
        ConstellationIdentifier {
            constellation_id: "".to_string(),
            node_id: "".to_string(),
        }
    }

    pub fn to_string(&self) -> String {
        String::from(
            format!("CID:{}:{}", self.constellation_id, self.node_id)
        )
    }
}