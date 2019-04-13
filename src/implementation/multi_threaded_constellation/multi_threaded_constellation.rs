use crate::constellation::ConstellationTrait;
use super::super::mpi::environment::Universe;
use crate::implementation::error::ConstellationError;
use crate::activity::ActivityTrait;
use crate::context::Context;
use crate::activity_identifier::ActivityIdentifier;
use crate::event::Event;
use crate::constellation_identifier::ConstellationIdentifier;
use crate::implementation::communication::mpi_info;
use crate::constellation_config::ConstellationConfiguration;
use crate::implementation::thread_helper::{MultiThreadHelper, ExecutorQueues, ThreadHelper};
use crate::implementation::single_threaded_constellation::inner_constellation::InnerConstellation;

use std::sync::{Arc, Mutex};
use std::thread;

use crossbeam::{Sender, Receiver, unbounded, deque};
use std::time;

pub struct MultiThreadedConstellation {
    thread_handler: Option<MultiThreadHelper>,
    signal_thread_handler: Option<(Sender<bool>, Receiver<bool>)>,
    universe: Universe,
    debug: bool,
    thread_count: i32,
    config: Box<ConstellationConfiguration>,
}

impl ConstellationTrait for MultiThreadedConstellation {
    /// Activate the MultiThreadedConstellation instance
    ///
    /// This will setup all the ExecutorThreads and the InnerConstellation types,
    /// and share necessary references between them.
    ///
    /// # Returns
    /// * `Result<bool, ConstellationError>` - A Result type containing a
    /// boolean which will have the value true if this is the master thread and
    /// false otherwise.
    ///
    /// Upon failure a ConstellationError will be returned
    fn activate(&mut self) -> Result<bool, ConstellationError> {
        if self.is_master().unwrap() {
            if self.debug {
                info!("Activating Multi Threaded Constellation");
            }

            // Queues used for threads to share events/activities with thread handler
            let activities_from_threads= Arc::new(Mutex::new(deque::Injector::new()));
            let events_from_threads= Arc::new(Mutex::new(deque::Injector::new()));

            // Shared between all threads, to generate unique activity IDs
            let activity_counter= Arc::new(Mutex::new(0));

            let mut thread_handler = MultiThreadHelper::new(self.debug, activities_from_threads.clone(), events_from_threads.clone());

            for i in 0..self.thread_count {
                let executor_queues = ExecutorQueues::new(&self.universe, activity_counter.clone(), i);

                // This struct links the activities and events passed through the functions "submit" and "send" to the thread_handler
                let helper = ThreadHelper::new(activities_from_threads.clone(), events_from_threads.clone());

                let inner_constellation: Arc<Mutex<Box<dyn ConstellationTrait>>> = Arc::new(Mutex::new(Box::new(InnerConstellation::new_multithreaded(
                    &self.config,
                    executor_queues.const_id.clone(),
                    helper,
                    executor_queues.activities.clone(),
                    executor_queues.activities_suspended.clone(),
                    executor_queues.event_queue.clone(),
                    i,
                ))));

                if let Some(inner) = inner_constellation
                    .lock()
                    .unwrap()
                    .downcast_mut::<InnerConstellation>() {
                    inner.activate_inner(inner_constellation.clone());
                }


                thread_handler.push(executor_queues, inner_constellation.clone());
            }

            // TODO Store these inside multi_threaded_constellation as well
            let (s, r): (Sender<bool>, Receiver<bool>) = unbounded();
            let (s2, r2): (Sender<bool>, Receiver<bool>) = unbounded();


            let mut inner_handler = thread_handler.clone();


            // Start multi thread handler, this function will periodically
            // check for new activities/events, try to steal events from other nodes
            // and perform load-balancing.
            thread::spawn(move || {
                inner_handler.run(r, s2);
            });

            self.thread_handler = Some(thread_handler);
            self.signal_thread_handler = Some((s, r2));

            return Ok(true);
        }

        Ok(false)
    }

    fn submit(
        &mut self,
        activity: Arc<Mutex<ActivityTrait>>,
        context: &Context,
        may_be_stolen: bool,
        expects_events: bool,
    ) -> ActivityIdentifier {
        self.thread_handler.as_mut().unwrap().submit(activity, context, may_be_stolen, expects_events)
    }

    fn send(&mut self, e: Box<Event>) {
        self.thread_handler.as_mut().unwrap().send(e);
    }

    fn done(&mut self) -> Result<bool, ConstellationError> {
        if self.debug {
            info!("Attempting to shut down Constellation gracefully");
        }

        let inner = self.thread_handler.as_mut().unwrap().done();

        if inner.is_ok() {
            info!("All threads were shutdown successfully");

            // All threads were shutdown ok
            if *inner.as_ref().unwrap() {
                // Shut down thread_handler
                self.signal_thread_handler.as_ref().unwrap().0.send(true).expect("Failed to send signal to load balancer");

                let time = time::Duration::from_secs(100);
                if self.debug {
                    info!("Waiting for {}s for load balancer to shut down", 100);
                }
                if let Ok(r) = self.signal_thread_handler.as_ref().unwrap().1.recv_timeout(time) {
                    if !r {
                        warn!("Something went wrong shutting down the load balancer");
                        return Err(ConstellationError);
                    }
                } else {
                    warn!("Timeout waiting for the load balancer to shutdown");
                    return Err(ConstellationError);
                }
                info!("Load balancer successfully shutdown");
            }
        }

        inner
    }

    fn identifier(&mut self) -> ConstellationIdentifier {
        unimplemented!();
    }

    fn is_master(&mut self) -> Result<bool, ConstellationError> {
        Ok(mpi_info::master(&self.universe))
    }

    fn nodes(&mut self) -> i32 {
        mpi_info::size(&self.universe)
    }

    fn set_parent(&mut self, _parent: Arc<Mutex<Box<dyn ConstellationTrait>>>){
        unimplemented!();
    }
}

impl MultiThreadedConstellation {
    pub fn new(config: Box<ConstellationConfiguration>) -> MultiThreadedConstellation {
        MultiThreadedConstellation {
            thread_handler: None,
            signal_thread_handler: None,
            universe: mpi::initialize().unwrap(),
            debug: config.debug,
            thread_count: config.number_of_threads,
            config
        }
    }
}
