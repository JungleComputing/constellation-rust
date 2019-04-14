///! An identifier for each thread running in constellation. It holds
///! information about all nodes and threads, as well as helps with generating
///! unique IDs for all newly submitted activities.

use mpi::environment::Universe;
use mpi::topology::{Communicator, Rank};

use std::collections::HashMap;
use std::fmt;
use std::sync::{Mutex, Arc};

use crate::implementation::communication::node_handler;

/// This struct is used to identify a certain thread and node in the running
/// Constellation instance. Each struct shares an Arc to a counter, which
/// should be used when generating new activities, in order to make them unique
/// across all threads/nodes.
///
/// # Members
/// * `constellation_id` - An i32 number identifying this entire constellation
/// instance. Can be used for e.g. distinguishing multiple executions of the
/// same program, by each time incrementing this number and saving the logs.
/// * `node_info` - NodeHandler struct, containing information about the node
/// which created this ConstellationIdentifier instance.
/// * `group` - A HashMap linking each MPI Rank to a certain NodeHandler struct,
/// used in order to quickly find node information for each process.
/// * `thread_id` - A number identifying the thread who created this instance
/// * `activity_counter` A shared Arc counter for all ConstellationIdentifier
/// instances, used to create unique IDs for all generated activities.
#[derive(Debug)]
pub struct ConstellationIdentifier {
    pub constellation_id: i32,
    pub node_info: node_handler::NodeHandler,
    pub group: HashMap<Rank, node_handler::NodeHandler>, // All processes and their node information
    pub thread_id: i32,
    pub activity_counter: Arc<Mutex<u64>>, // Shared between all threads
}

impl ConstellationIdentifier {
    /// Generate a new ConstellationIdentifier which contains an unique ID for
    /// this constellation instance, information about how many nodes/threads
    /// there are as well as the thread which is
    /// "currently running with this ID".
    ///
    /// # Arguments
    /// * `universe` - MPI Universe construct
    /// * `activity_counter` - An Arc<Mutex<u64>> counter, which is used to
    /// keep all ActivityIdentifiers unique across the entire constellation
    /// instance. Always increment this counter when creating a new activity ID
    /// * `thread_id` - A unique number identifying each thread
    ///
    /// # Returns
    /// * `ConstellationIdentifier` - Unique ConstellationIdentifier
    /// for each thread on each node
    pub fn new(universe: &Universe, activity_counter: Arc<Mutex<u64>>, thread_id: i32) -> ConstellationIdentifier {
        let world = universe.world();
        let rank = world.rank();

        let mut const_id = ConstellationIdentifier {
            constellation_id: 0,
            node_info: node_handler::NodeHandler {
                node_name: mpi::environment::processor_name()
                    .expect("Could not retrieve processor_name"),
                node_id: 0,
            },
            group: HashMap::new(),
            thread_id,
            activity_counter,
        };

        // Create mpi groups to track processes on each node
        node_handler::create_groups(&mut const_id.group, &universe);

        const_id.node_info.node_id = const_id.group.get(&rank).unwrap().node_id;

        const_id
    }


    /// Create a new empty ConstellationIdentifier.
    /// This one still needs to get node_info and group set.
    ///
    /// # Returns
    /// * `ConstellationIdentifier` - A new, possibly NOT unique,
    /// constellationIdentifier
    pub fn new_empty() -> ConstellationIdentifier {
        ConstellationIdentifier {
            constellation_id: 0,
            node_info: node_handler::NodeHandler {
                node_name: "EMPTY".to_string(),
                node_id: 0,
            },
            group: HashMap::new(),
            thread_id: 0,
            activity_counter: Arc::new(Mutex::new(0)),
        }
    }

    /// Increment the counter when creating a unique number for an activity
    ///
    /// # Returns
    /// * `u64` - A unique number which can be used in an ActivityIdentifier
    pub fn generate_activity_id(&mut self) -> u64 {
        let mut guard = self.activity_counter.lock().unwrap();

        let ret = guard.clone();
        *guard += 1;

        drop(guard);
        ret
    }
}

impl fmt::Display for ConstellationIdentifier {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "CID:{}:NID:{}:TID:{}",
            self.constellation_id, self.node_info.node_id, self.thread_id
        )
    }
}

impl Clone for ConstellationIdentifier {
    fn clone(&self) -> Self {
        ConstellationIdentifier {
            constellation_id: self.constellation_id.clone(),
            node_info: self.node_info.clone(),
            group: HashMap::new(),
            thread_id: self.thread_id,
            activity_counter: self.activity_counter.clone(),
        }
    }
}
