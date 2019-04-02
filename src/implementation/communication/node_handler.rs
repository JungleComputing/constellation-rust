extern crate mpi;

use mpi::collective::CommunicatorCollectives;
use mpi::datatype::PartitionMut;
use mpi::environment::Universe;
use mpi::topology::{Communicator, Rank};
use mpi::Count;
use std::collections::HashMap;

/// Store node information
///
/// * `node_name` - The processor name, can be received with
/// mpi::environment::processor_name() from one of the processes running on the node
/// * `node_id` - An unique identifier for this node
#[derive(Debug, Clone, Hash)]
pub struct NodeHandler {
    pub node_name: String,
    pub node_id: usize,
}

/// Store in the provided HashMap which process runs on which node, data
/// is stored using the NodeHandler struct.
///
/// This method MUST be called from each MPI process.
///
/// # Arguments
/// * `&mut HashMap` - The HashMap to add node information to to, will be
/// updated in place
/// * `&Universe` - The Universe object from MPI, upon which MPI has already
///  been initialized
pub fn create_groups(groups: &mut HashMap<Rank, NodeHandler>, universe: &Universe) {
    let world = universe.world();
    let size = world.size();
    let process: Vec<u8> =
        Vec::from(mpi::environment::processor_name().expect("Could not retrieve processor name"));

    // Gather the length of all the strings that will be sent
    let msg = process.len();
    let mut all_lengths = vec![0u64; size as usize];
    world.all_gather_into(&msg, &mut all_lengths[..]);

    let counts: Vec<Count> = all_lengths.iter().map(|&x| x as i32).collect();
    let mut displs: Vec<Count> = {
        let mut temp_v = Vec::new();
        temp_v.push(0);
        for i in 1..size {
            temp_v.push(counts[i as usize]);
        }
        temp_v
    };

    // All characters (bytes) will be stored in this array, where each nodes
    // starting point is indicated by the corresponding index in displs
    let mut result: Vec<u8> = vec![
        0;
        all_lengths.iter().fold(0, |mut sum, &x| {
            sum += x as usize;
            sum
        })
    ];

    let mut partition = PartitionMut::new(&mut result[..], counts, &displs[..]);

    // Gather all node names
    world.all_gather_varcount_into(&process[..], &mut partition);

    displs.push(result.len() as i32);
    // Add collected data to HashMap
    for i in 0..size as usize {
        let name = &result[displs[i] as usize..displs[i + 1] as usize];
        let name_string = String::from_utf8(Vec::from(name)).unwrap();
        groups.insert(
            i as i32,
            NodeHandler {
                node_name: name_string,
                node_id: i as usize,
            },
        );
    }
}
