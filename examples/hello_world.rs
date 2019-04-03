//! Create two activities and send a Hello World payload from one to the other.

extern crate constellation_rust;

use std::env;
use std::fmt;
use std::sync::{Arc, Mutex};

use constellation_rust::activity_identifier::ActivityIdentifier;
use constellation_rust::constellation::ConstellationTrait;
use constellation_rust::constellation_config;
use constellation_rust::constellation_factory::{new_constellation, Mode};
use constellation_rust::context::Context;
use constellation_rust::event::Event;
use constellation_rust::payload::{PayloadTrait, PayloadTraitClone};
use constellation_rust::{activity, activity::ActivityTrait};
use constellation_rust::{steal_strategy, SingleEventCollector};

/// Payload struct for passing data between activities
/*---------------------------------------------------------------------------*/
#[derive(Debug, Clone)]
struct Payload {
    data: String,
}

impl PayloadTrait for Payload {}

impl PayloadTraitClone for Payload {
    fn clone_box(&self) -> Box<dyn PayloadTrait> {
        Box::new(self.clone())
    }
}

impl fmt::Display for Payload {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.data)
    }
}

/// Activity which only sends an event containing the payload "hello world"
/// to another specified activity
/*---------------------------------------------------------------------------*/
struct HelloWorldActivity {
    target: ActivityIdentifier,
}

impl ActivityTrait for HelloWorldActivity {
    fn cleanup(&mut self, _constellation: Arc<Mutex<Box<dyn ConstellationTrait>>>) {
        // no cleanup necessary
    }

    fn initialize(
        &mut self,
        constellation: Arc<Mutex<Box<dyn ConstellationTrait>>>,
        id: &ActivityIdentifier,
    ) -> activity::State {
        // Create an event and send it to process with id self.target
        let msg = Payload {
            data: "Hello World".to_string(),
        };

        let event = Event::new(Box::from(msg), id.clone(), self.target.clone());

        // Send the event containing the payload string
        constellation
            .lock()
            .expect("Could not get lock on Constellation instance")
            .send(event);

        return activity::State::FINISH;
    }

    fn process(
        &mut self,
        _constellation: Arc<Mutex<Box<dyn ConstellationTrait>>>,
        _event: Option<Box<Event>>,
        _id: &ActivityIdentifier,
    ) -> activity::State {
        // No process necessary
        return activity::State::FINISH;
    }
}
/*---------------------------------------------------------------------------*/

/// Create two activities, one which sends a "payload" type containing the
/// string "Hello World" to anther activity, which only purpose is to wait for
/// the payload and then return it to this program, in order to be displayed.
///
/// # Arguments
/// * `constellation` - A boxed Constellation instance
fn run(mut constellation: Box<dyn ConstellationTrait>) {
    let master = constellation
        .is_master()
        .expect("Error when checking if current node is master");

    // Only submit activities from one thread.
    if master {
        let context = Context {
            label: "HelloContext".to_string(),
        };

        let sec = SingleEventCollector::new();

        // When submitting activity we need to cast the SingleEventCollector to
        // be of the trait type ActivityTrait
        let sec_aid = constellation.submit(
            sec.clone() as Arc<Mutex<ActivityTrait>>,
            &context,
            false,
            true,
        );

        let hello_activity: Arc<Mutex<ActivityTrait>> =
            Arc::new(Mutex::new(HelloWorldActivity { target: sec_aid }));

        constellation.submit(hello_activity, &context, true, false);

        println!("Both events submitted to Constellation");

        let time = std::time::Duration::from_secs(1);

        println!("Waiting for payload in SingleEventCollector...");
        let e = SingleEventCollector::get_event(sec, time);

        println!("Got payload! Shutting down Constellation");

        // Shut down constellation gracefully
        constellation.done().expect(
            "Failed to shutdown constellation"
        );

        println!(
            "\n-----------------------------------------------------------\
            \nSRC activity ID: {}\
             \nDST activity ID: {}\nPayload: {}\
             \n-----------------------------------------------------------",
            e.get_src(),
            e.get_dst(),
            e.get_payload(),
        );
    }
}

/// Main function takes one argument, specifying the number of nodes to use.
/// It creates a constellation configuration with only steal strategy biggest,
/// and the activities are very minimalistic.
fn main() {
    // Retrieve user arguments
    let args: Vec<String> = env::args().collect();

    let nmr_nodes = args[1].parse().expect(&format!(
        "Cannot parse {} into an integer, please provide number of nodes",
        args[1]
    ));

    let const_config = constellation_config::ConstellationConfiguration::new(
        steal_strategy::BIGGEST,
        steal_strategy::BIGGEST,
        steal_strategy::BIGGEST,
        nmr_nodes,
        true,
    );

    let mut constellation = new_constellation(Mode::SingleThreaded, const_config);

    constellation.activate().unwrap();

    run(constellation);
}
