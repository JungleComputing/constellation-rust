//! Prints hello worlds on all nodes, including their name and id

extern crate constellation_rust;

use constellation_rust::constellation::ConstellationTrait;
use constellation_rust::constellation_config;
use constellation_rust::context::Context;
use constellation_rust::event::Event;
use constellation_rust::single_threaded_constellation::SingleThreadConstellation;
use constellation_rust::steal_strategy;
use constellation_rust::{activity, activity::ActivityTrait};
use std::env;
use std::process::exit;

use constellation_rust::SingleEventCollector;

const LABEL: &str = "Hello World";

struct HelloWorldActivity {}

impl ActivityTrait for HelloWorldActivity {
    fn cleanup(&self, constellation: &ConstellationTrait) {
        // no cleanup necessary
    }

    fn initialize(&self, constellation: &ConstellationTrait) -> usize {
        // Don't process anything, just suspend for later processing
        return activity::FINISH;
    }

    fn process(&self, constellation: &ConstellationTrait, event: Event) -> usize {
        // No process necessary
        println!("{}", event.get_message());
        return activity::FINISH;
    }
}

fn run(constellation: &mut SingleThreadConstellation) {
    let master = constellation
        .is_master()
        .expect("Error when checking if current node is master");

    if master {
        let activity = Box::from(HelloWorldActivity {});
        let context = Context {
            label: LABEL.to_string(),
        };

        // Wait for 1 event and print it
        let sec = SingleEventCollector::new();
        constellation.submit(sec, &context, false, true);

        //constellation.submit(activity, context, true, true);
    }
}

fn main() {
    let mut master: bool;

    // Retrieve user arguments
    let args: Vec<String> = env::args().collect();

    let nmr_nodes = args[1].parse().expect(&format!(
        "Cannot parse {} into an integer, please provide number of nodes",
        args[1]
    ));

    let const_config = constellation_config::ConstellationConfiguration::new_all(
        steal_strategy::BIGGEST,
        steal_strategy::BIGGEST,
        steal_strategy::BIGGEST,
        nmr_nodes,
    );

    let mut constellation = SingleThreadConstellation::new(const_config);

    constellation.activate();

    run(&mut constellation);
}
