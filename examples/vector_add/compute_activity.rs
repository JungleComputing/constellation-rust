///! Activity which receives the two arrays to add to each other from an event,
///! checks whether they are smaller than the given threshold, if yes, computes
///! the result and returns it. If no, create two new activities and pass them
///! each half of the array.
///!
///! The process described above will solve the vector addition using divide
///! and conquer, distributed with constellation.
///
use std::sync::{Arc, Mutex};

use constellation_rust::activity;
use constellation_rust::activity_identifier::ActivityIdentifier;
use constellation_rust::constellation::ConstellationTrait;
use constellation_rust::context::Context;
use constellation_rust::event::Event;

use super::context::CONTEXT;
use super::payload;

pub struct ComputeActivity {
    pub vec1: Vec<i32>,
    pub vec2: Vec<i32>,
    pub threshold: i32,
    pub target: ActivityIdentifier,
    pub order: Option<(ActivityIdentifier, ActivityIdentifier)>,
    pub waiting_for_event: bool,
}

impl activity::ActivityTrait for ComputeActivity {
    fn cleanup(&mut self, _constellation: Arc<Mutex<Box<dyn ConstellationTrait>>>) {
        // no cleanup necessary
    }

    /// Initialize method from trait object.
    ///
    /// This method will always be called first on the activity. If the
    /// vectors are shorter than the threshold, calculate the result and return
    /// it to the parent. Otherwise split the problem over two new activities.
    ///
    /// # Arguments
    /// * `constellation` - Arc reference to the Constellation instance
    /// * `id` - Identifier for this activity
    ///
    /// # Returns
    /// * `State` - The state of which to put the activity in when returning
    fn initialize(
        &mut self,
        constellation: Arc<Mutex<Box<dyn ConstellationTrait>>>,
        id: &ActivityIdentifier,
    ) -> activity::State {
        // Check if length is smaller than threshold
        if self.vec1.len() < self.threshold as usize {
            let result = self.compute_result();

            self.send_result_to_parent(constellation, &id, result);
        } else {
            self.split_work_over_children(constellation, &id);
            self.waiting_for_event = true;
        }

        return activity::State::FINISH;
    }

    /// Process method from trait object.
    ///
    /// This method will in the case of our program only be called in 2
    /// situations:
    ///     - The initialize method returned State::FINISH
    ///     - The activity is suspended but received and Event
    ///
    /// The second situation indicates that a child process is returning the
    /// result of solving a sub problem.
    ///
    /// # Arguments
    /// * `constellation` - Àrc reference to the Constellation instance.
    /// * `event` - An event type wrapped in a option<..>, possibly containing
    /// the result of solving a sub problem.
    /// * `id` - Identifier for this activity
    ///
    /// # Returns
    /// * `State` - The state of which to put the activity in when returning
    fn process(
        &mut self,
        constellation: Arc<Mutex<Box<dyn ConstellationTrait>>>,
        event: Option<Box<Event>>,
        id: &ActivityIdentifier,
    ) -> activity::State {

        // Check if we got activated because of receiving an event,
        // this indicates that the children are done processing
        if let Some(e) = event {
            return self.process_event(constellation, e, id);
        }

        if self.waiting_for_event {
            return activity::State::SUSPEND;
        }

        return activity::State::FINISH;
    }
}

impl ComputeActivity {

    /// Compute the result when adding the two vectors together.
    ///
    /// # Returns
    /// * `Vec<i32>` - The resulting vector after addition
    pub fn compute_result(&mut self) -> Vec<i32> {
        self.vec1
            .iter()
            .zip(self.vec2.iter())
            .map(|(x, y)| x + y)
            .collect()
    }

    /// Send the result to the parent activity
    ///
    /// # Arguments
    /// * `constellation` - Arc reference to the Constellation instance
    /// * `id` - Identifier for this activity
    /// * `vec` - The resulting vector after addition
    pub fn send_result_to_parent(
        &mut self,
        constellation: Arc<Mutex<Box<dyn ConstellationTrait>>>,
        id: &ActivityIdentifier,
        vec: Vec<i32>,
    ) {
        // Create an event and send it to process with id self.target
        let msg = payload::Payload { vec };

        let event = Event::new(Box::from(msg), id.clone(), self.target.clone());

        // Send the event containing the payload string
        constellation
            .lock()
            .expect("Could not get lock on Constellation instance")
            .send(event);
    }

    /// Split the vectors over two new activities and wait for an event containing
    /// the result from each child.
    ///
    /// # Arguments
    /// * `constellation` - Arc reference to the Constellation instance
    /// * `id` - Identifier for this activity
    pub fn split_work_over_children(
        &mut self,
        constellation: Arc<Mutex<Box<dyn ConstellationTrait>>>,
        id: &ActivityIdentifier,
    ) {
        // Create two new activities and split the work between them and one
        // activity waiting for events
        let half = (self.vec1.len() / 2) as i32;
        let a: Arc<Mutex<activity::ActivityTrait>> = Arc::new(Mutex::new(ComputeActivity {
            vec1: Vec::from(&self.vec1[0..half as usize]),
            vec2: Vec::from(&self.vec1[0..half as usize]),
            threshold: self.threshold,
            target: id.clone(), // Make child send event to this activity
            order: None,
            waiting_for_event: false
        }));

        let b: Arc<Mutex<activity::ActivityTrait>> = Arc::new(Mutex::new(ComputeActivity {
            vec1: Vec::from(&self.vec1[half as usize..(self.vec1.len() as i32) as usize]),
            vec2: Vec::from(&self.vec1[half as usize..(self.vec2.len() as i32) as usize]),
            threshold: self.threshold,
            target: id.clone(), // Make child send event to this activity
            order: None,
            waiting_for_event: false
        }));

        // Acquire lock on constellation
        let mut guard = constellation.lock().expect(&format!(
            "Could not get lock on constellation from \
             inside executing activity: {}",
            id
        ));

        // Submit compute activities to constellation
        let aid_1 = guard.submit(
            a,
            &Context {
                label: String::from(CONTEXT),
            },
            true,
            false,
        );
        let aid_2 = guard.submit(
            b,
            &Context {
                label: String::from(CONTEXT),
            },
            true,
            false,
        );

        drop(guard);

        // Use vec1 for storing the result received from children
        self.vec1 = Vec::new();

        // Set the order in which to the children result must be stored
        self.order = Some((aid_1, aid_2));
    }

    /// Process a received event, by first checking if this was the first or
    /// second received event and stitching them together in the correct order.
    ///
    /// The payload received in the Event is cast to match the self-made
    /// payload type
    ///
    /// # Arguments
    /// * `constellation` - Arc reference to the Constellation instance
    /// * `event` - The received event
    /// * `id` - Identifier for this activity
    ///
    /// # Returns
    /// * `State` - The state of which to put the activity in when returning
    fn process_event(
        &mut self,
        constellation: Arc<Mutex<Box<dyn ConstellationTrait>>>,
        event: Box<Event>,
        id: &ActivityIdentifier,
    ) -> activity::State {
        let mut v: Vec<i32> = event
            .get_payload()
            .downcast_ref::<payload::Payload>()
            .unwrap()
            .vec
            .clone();

        // Check if this is the first event received
        if self.vec1.len() == 0 {
            self.vec1 = v;
            // Waiting for 1 more event
            return activity::State::SUSPEND;
        }

        if self.order.as_mut().unwrap().0 == event.get_src() {
            v.append(&mut self.vec1);
            self.vec1 = v;
        } else {
            self.vec1.append(&mut v);
        }

        // Send result back to parent
        self.send_result_to_parent(constellation, &id, self.vec1.clone());

        return activity::State::FINISH;
    }
}
