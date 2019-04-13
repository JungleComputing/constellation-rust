//! An example program of how to perform vector addition using Constellation.
//!
//! This program uses a divide and conquer approach for the solution,
//! recursively creating new activities to solve the sub problems.

extern crate constellation_rust;

use std::env;
use std::process::exit;
use std::sync::{Arc, Mutex};

use constellation_rust::constellation::ConstellationTrait;
use constellation_rust::constellation_factory::{new_constellation, Mode};
use constellation_rust::context::Context;
use constellation_rust::context::ContextVec;
use constellation_rust::{activity, SingleEventCollector};
use constellation_rust::{constellation_config, steal_strategy};
use std::time::Instant;

mod compute_activity;
mod context;
mod payload;

const THRESHOLD: i32 = 10;

/// Creates a SingleEventCollector and a ComputeActivity will will be the
/// base of the vector add.
///
/// The SingleEventCollector will simply wait for an
/// event containing the final result, and the ComputeActivity will start
/// calculating this result in a divide and conquer type of way, with a
/// threshold set by the user.
///
/// # Arguments
/// * `constellation` - Constellation instance
/// * `vec1` - First vector used in the addition
/// * `vec2` - Second vector used in the addition
///
/// # Returns
/// * `Vec<i32>` - The resulting vector
fn constellation_vector_add(
    constellation: &mut Box<dyn ConstellationTrait>,
    vec1: Vec<i32>,
    vec2: Vec<i32>,
) -> Vec<i32> {
    assert_eq!(
        vec1.len(),
        vec2.len(),
        "Vector1 and vector2 are not the same length: {} - {}",
        vec1.len(),
        vec2.len()
    );

    // Create a single event collector to collect the final result
    let sec = SingleEventCollector::new();
    let sec_aid = constellation.submit(
        sec.clone() as Arc<Mutex<activity::ActivityTrait>>,
        &Context {
            label: String::from(context::CONTEXT),
        },
        false,
        true,
    );

    // This activity will be the base of all calculation
    let start_compute_activity: Arc<Mutex<activity::ActivityTrait>> =
        Arc::new(Mutex::new(compute_activity::ComputeActivity {
            vec1,
            vec2,
            threshold: THRESHOLD,
            target: sec_aid,
            order: None,
            waiting_for_event: false,
        }));

    constellation.submit(
        start_compute_activity,
        &Context {
            label: String::from(context::CONTEXT),
        },
        true,
        false,
    );

    // Wait for result
    let time = std::time::Duration::from_secs(1);
    let e = SingleEventCollector::get_event(sec, time);

    let result = e
        .get_payload()
        .downcast_ref::<payload::Payload>()
        .unwrap()
        .vec
        .clone();

    result
}

/// Create vectors deepening on array_length and print result after execution
///
/// # Arguments
/// * `constellation` - Constellation instance
/// * `array_length` - User defined length of the vectors used in the addition
fn run(mut constellation: Box<dyn ConstellationTrait>, array_length: i32) {
    println!("Running Vector add with {} nodes", constellation.nodes());

    // Create two vectors and fill
    // them with incrementing values from 0..<array_length>
    let mut vec1 = Vec::new();
    let mut vec2 = Vec::new();

    for x in 0..array_length {
        vec1.push(x);
        vec2.push(x);
    }

    let result = constellation_vector_add(&mut constellation, vec1, vec2);
     // Shut down constellation gracefully
    constellation
        .done()
        .expect("Failed to shutdown constellation");

    let length = if array_length < 40 { array_length } else { 40 };
    println!(
        "\n--------------------------------------------------------\
     \nThe first {} elements in the resulting array are:\n{:?}\
     \n--------------------------------------------------------",
        length,
        &result[0..length as usize]
    );
}

/// Gathers user input and creates a constellation instance.
fn main() {
    // Retrieve user arguments
    let args: Vec<String> = env::args().collect();
    if args.len() < 4 {
        println!(
            "Please provide an number of nodes and array length\n\
             mpirun ARGS vector_add <nmr_nodes> <nmr_threads> <array_length>"
        );
        exit(1);
    }

    let nmr_nodes = args[1].parse().expect(&format!(
        "Cannot parse {} into an integer, please provide number of nodes",
        args[1]
    ));

    let nmr_threads = args[2].parse().expect(&format!(
        "Cannot parse {} into an integer, please provide number of nodes",
        args[1]
    ));

    let array_length = args[3].parse().expect(&format!(
        "Cannot parse {} into an integer, please provide a valid array length",
        args[2]
    ));

    let mut context_vec = ContextVec::new();
    context_vec.append(&Context {
        label: String::from(context::CONTEXT),
    });

    let const_config = constellation_config::ConstellationConfiguration::new(
        steal_strategy::BIGGEST,
        steal_strategy::BIGGEST,
        steal_strategy::BIGGEST,
        nmr_nodes,
        nmr_threads,
        true,
        context_vec,
    );

    let mut constellation = new_constellation(Mode::MultiThreaded, const_config);

    constellation.activate().unwrap();

    if constellation.is_master().unwrap() {
        // Execute vector add for the given length on the constellation instance
        let now = Instant::now();
        run(constellation, array_length);
        println!("\n\nExecution took: {}s", now.elapsed().as_secs());
    }
}
