//! Prints hello worlds on all nodes, including their name and id

extern crate constellation_rust;

use constellation_rust::constellation::ConstellationTrait;
use constellation_rust::constellation_config;
use constellation_rust::context::Context;
use constellation_rust::event::Event;
use constellation_rust::constellation_factory::{Mode, new_constellation};
use constellation_rust::steal_strategy;
use constellation_rust::{activity, activity::ActivityTrait};
use std::env;
use std::process::exit;

use constellation_rust::SingleEventCollector;
use constellation_rust::activity_identifier::ActivityIdentifier;
use std::sync::Mutex;
use std::sync::Arc;
use constellation_rust::message::MessageTrait;
use std::time;
use std::thread;

const LABEL: &str = "Hello World";

struct HelloWorldActivity {
    target: ActivityIdentifier,
}

struct Message {
    data: String,
}

impl MessageTrait for Message {
    fn to_string(&self) -> &String {
        &self.data
    }
}

impl ActivityTrait for HelloWorldActivity {
    fn cleanup(&mut self,  constellation: Arc<Mutex<Box<dyn ConstellationTrait>>>) {
        // no cleanup necessary
    }

    fn initialize(&mut self, constellation: Arc<Mutex<Box<dyn ConstellationTrait>>>) -> usize {
        // Create an event and send it to process with id self.target

        let msg = Message {
            data: LABEL.to_string(),
        };

        let event = Event::new(Box::from(msg));

        // Send the event containing the message string
        constellation.lock().expect(
            "Could not get lock on Constellation instance"
        ).send(event);

        return activity::FINISH;
    }

    fn process(&mut self, constellation: Arc<Mutex<Box<dyn ConstellationTrait>>>, event: Event) -> usize {
        // No process necessary
        return activity::FINISH;
    }
}

fn run(mut constellation: Box<dyn ConstellationTrait>) {
    let master = constellation
        .is_master()
        .expect("Error when checking if current node is master");

    if master {
        let context = Context {
            label: LABEL.to_string(),
        };

        let sec = SingleEventCollector::new();
        let sec_aid = constellation.submit(
            &sec, &context, false, true);

        let hello_activity: Arc<Mutex<ActivityTrait>> = Arc::new(Mutex::new(HelloWorldActivity {
            target: sec_aid,
        }));

        constellation.submit(&hello_activity, &context, true, false);

        println!("Both events submitted to Constellation");
        let time = time::Duration::from_secs(3);
        thread::sleep(time);
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

    let mut constellation = new_constellation(Mode::SingleThreaded, const_config);

    constellation.activate();

    run(constellation);
}
