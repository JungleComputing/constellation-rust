///
/// Implementation of vector add in order to test Constellation during development
/// This package should not be an executable when finished, but a library.

extern crate mpi;

use mpi::traits::*;
use mpi::Count;
use mpi::topology::SystemCommunicator;
use mpi::datatype::PartitionMut;
use std::env;
use std::process::exit;

use std::ops::{Add};

/// Structure holding a vector with any type of elements.
#[derive(Debug, PartialEq)]
pub struct Evec<T> {
    pub vec: Vec<T>,
}

/// Add two Evec structures together by internally performing a
/// vector addition
///
/// '''
/// # Examples
///
/// let mut vec1: Evec<i32> = Evec<i32>;
/// let mut vec2: Evec<i32> = Evec::new();
///
/// vec1.vec.push(1);
/// vec2.vec.push(2);
///
/// let mut vec3 = vec1 + vec2;
///
/// assert_eq!(vec3, [3]);
/// '''
impl<T> Add<Evec<T>> for Evec<T>
    where T: Add<Output=T> + Copy {

    type Output = Evec<T>;

    fn add(self, other: Evec<T>) -> Evec<T> where T: Copy {
        let shortest: usize;

        if self.vec.len() < other.vec.len() {
            shortest = self.vec.len();
        } else {
            shortest = other.vec.len();
        }

        let mut new_vec = Evec::new();

        for x in 0..shortest {
            let value= self.vec[x] + other.vec[x];
            new_vec.vec.push(value);
        }

        return new_vec;
    }
}

impl <T> Evec<T>{
    /// Generate a new Evec<T>
    pub fn new() -> Evec<T>{
        Evec { vec: Vec::new()}
    }

    /// Get the length of the internal vector
    pub fn size(&self) -> usize{
        self.vec.len()
    }

    /// Compare to another Evec 'other' to check if internal
    /// vector is smaller than the one from 'other'
    pub fn smaller(&self, other: &Evec<T>) -> bool{
        if self.size() < other.size() {
            return true;
        }

        return false;
    }

    /// Get the length of the inner vector
    pub fn len(self) -> usize {
        return self.vec.len();
    }
}

/// Perform vector in a distributed setting, using Constellation
///
/// Will only use the number of elements equal to the length of the smallest array, excess
/// elements will be dropped.
fn distributed_vector_add(vec1: Evec<i32>, vec2: Evec<i32>) -> Evec<i32>{
    let smallest_len = if vec1.smaller(&vec2) {
        vec1.size() as i32
    } else {
        vec2.size() as i32
    };

    // Calculate interval for this node
    let start = (smallest_len * rank / size) as i32;
    let len = (smallest_len * (rank + 1) / size) as i32 - (smallest_len * rank / size) as i32;

    //TODO Create constillation instance and perform vector addition, result should
    //TODO be stored in vector buf and returned to main function

    Evec { vec: buf}
}

fn main() {
    let world = //TODO number of nodes running
    let master = //TODO elect one node to be 'master'

    let args: Vec<String> = env::args().collect();
    if args.len() < 2 {
        if root {
            println!("Please provide an array length:\n\
            Usage: mpirun ARGS vector_add.rs <array_length>\n---------------------\n");
        }
        exit(1);
    }

    let array_length = args[1].parse().unwrap();

    // Create two vectors and fill them with incrementing values from 0..<user_input>
    let mut vec1 = Evec::new();
    let mut vec2 = Evec::new();

    for x in 0..array_length {
        vec1.vec.push(x);
        vec2.vec.push(x);
    }

    if root {
//        println!("Running distributed vector add on {} nodes.",
//                 world.size());
        //TODO Print number of total nodes used
    }

    let result = distributed_vector_add(vec1, vec2);

    if root {
        let length = if array_length < 30 {array_length} else {30};
        println!("The first 30 elements in the resulting array are:\n{:?}", &result.vec[0..length as usize]);
    }
}
