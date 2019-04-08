//! Contains functions for all MPI information used in Constellation,
//! could be replaced with an alternative communication scheme

extern crate mpi;

use mpi::environment::Universe;
use mpi::topology::{Communicator, SystemCommunicator};

/// Get the MPI rank of the calling process
pub fn rank(universe: &Universe) -> i32 {
    universe.world().rank()
}

pub fn world(universe: &Universe) -> SystemCommunicator {
    universe.world()
}

pub fn size(universe: &Universe) -> i32 {
    universe.world().size()
}

pub fn master(universe: &Universe) -> bool {
    universe.world().rank() == 0
}
