//! Prints hello worlds on all nodes, including their name and id

extern crate constellation_rust;
use constellation_rust::single_threaded_constellation::SingleThreadConstellation;
use constellation_rust::constellation::ConstellationTrait;
use constellation_rust::steal_strategy;
use constellation_rust::constellation_config;
use constellation_rust::activity::ActivityTrait;
use std::env;
use std::process::exit;
use constellation_rust::event::Event;
use constellation_rust::context::Context;

const LABEL:String = "hello_world".to_string();

struct HelloWorldActivity {}

impl ActivityTrait for HelloWorldActivity {
    fn cleanup(&self, constellation: &ConstellationTrait) {
        // no cleanup necessary
    }

    fn initialize(&self, constellation: &ConstellationTrait) {
        // Don't process anything, just suspend for later processing
        ActivityTrait::SUSPEND;
    }

    fn process(&self, constellation: &ConstellationTrait, event: Event) {
        // Print hello world upon execution
        println!("{}", event.get_message());

        ActivityTrait::FINISH;
    }
}

fn run(constellation: SingleThreadConstellation) {
    let master = constellation.is_master().expect(
        "Error when checking if current node is master"
    );

    if master {
        let activity = HelloWorldActivity {};
        let context = Context { label: LABEL };

        let sec: SingleEventCollector = // Wait until hello_world has been printed

        println!("Running Vector add with {} nodes", constellation.nodes());

        constellation.submit_event(sec); //TODO IS THIS THE RIGHT WAY TO GO??

        constellation.submit(activity, context, true, true);


    }
}

fn main() {
    let mut master: bool;

    // Retrieve user arguments
    let args: Vec<String> = env::args().collect();

    let const_config = constellation_config::ConstellationConfiguration::new_all(
        steal_strategy::BIGGEST,
        steal_strategy::BIGGEST,
        steal_strategy::BIGGEST,
        nodes as usize,
    );

    let mut constellation = SingleThreadConstellation::new(const_config);

    constellation.activate();

    run(constellation);
}