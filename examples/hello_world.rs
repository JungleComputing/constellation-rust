//! Prints hello worlds on all nodes, including their name and id

extern crate constellation_rust;
use constellation_rust::single_threaded_constellation::SingleThreadConstellation;

fn main() {
    //TODO create constellation instance

    //TODO Print from each node

    let constellation = SingleThreadConstellation::new();

    println!("Hello World!");
}