///! Multithreaded constellation, should be initiated and used via the
///! ConstellationFactory type. The MultiThreadedConstellation initializes all
///! shared data structures and can be seen as a "wrapper class" returned to
///! the user defined application.
///!
///! The actual thread logic and work distribution is taken care of with the
///! thread_handler struct, this class only initializes everything and redirects
///! user called functions to the correct place in the handler
use super::super::mpi::environment::Universe;
use crate::implementation::communication::mpi_info;
use crate::implementation::constellation_files::inner_constellation::InnerConstellation;
use crate::implementation::constellation_files::thread_helper::{
    ExecutorQueues, MultiThreadHelper, ThreadHelper,
};
use crate::implementation::constellation_identifier::ConstellationIdentifier;
use crate::{
    ActivityIdentifier, ActivityTrait, ConstellationConfiguration, ConstellationError,
    ConstellationTrait, Context, Event,
};

use std::sync::{Arc, Mutex};
use std::thread;

use crossbeam::{deque, unbounded, Receiver, Sender};
use std::time;

/// Contains all the wrapper information necessary for the user to communicate
/// with the thread_handler and the InnerConstellation/Executor threads.
///
/// # Members
/// * `const_id` - ConstellationIdentifier type, unique for this instance. Will
/// have thread_id set to -1
/// * `thread_handler` - The struct handling load balancing, submission of
/// activities/events and inter-node communication
/// * `signal_thread_handler` - Tuple holding communicators to signal the
/// thread_handler, used for shutting down Constellation.
/// * `universe` - MPI universe struct
/// * `debug` - From configuration, used to determine whether to print debug
/// messages or not
/// * `thread_count` - Number of threads specified by user
/// * `config` - ConstellationConfiguration struct
pub struct MultiThreadedConstellation {
    const_id: ConstellationIdentifier,
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
    /// and share necessary references between them. It then creates the
    /// thread_handler and passes on all threads to this type, where all
    /// multithreaded logic will take place.
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
                info!("Activating Multithreaded Constellation");
            }

            // Queues used for threads to share events/activities with thread handler
            let activities_from_threads = Arc::new(Mutex::new(deque::Injector::new()));
            let events_from_threads = Arc::new(Mutex::new(deque::Injector::new()));

            let mut thread_handler = MultiThreadHelper::new(
                self.debug,
                activities_from_threads.clone(),
                events_from_threads.clone(),
                self.config.time_between_steals,
            );

            for i in 0..self.thread_count {
                let executor_queues =
                    ExecutorQueues::new(Arc::new(Mutex::new(ConstellationIdentifier::new(
                        &self.universe,
                        self.const_id.activity_counter.clone(),
                        i,
                    ))));

                // This struct links the activities and events passed through the functions "submit" and "send" to the thread_handler
                let helper =
                    ThreadHelper::new(activities_from_threads.clone(), events_from_threads.clone());

                let inner_constellation: Arc<Mutex<Box<dyn ConstellationTrait>>> =
                    Arc::new(Mutex::new(Box::new(InnerConstellation::new_multithreaded(
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
                    .downcast_mut::<InnerConstellation>()
                {
                    inner.activate_inner(inner_constellation.clone());
                }

                thread_handler.push(executor_queues, inner_constellation.clone());
            }

            let (s, r): (Sender<bool>, Receiver<bool>) = unbounded();
            let (s2, r2): (Sender<bool>, Receiver<bool>) = unbounded();

            let mut inner_handler = thread_handler.clone();

            // Start multi-thread handler, this function will periodically
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

    /// Submit a new activity from user application, redirects to the thread
    /// handler for load balancing.
    ///
    /// # Arguments
    /// * `activity` - A reference to an activity implementing the ActivityTrait.
    /// The activity must be inside an Arc<Mutex<..>>, in order to work with
    /// thread safety.
    /// * `context` - A reference to the context created for this activity
    /// * `may_be_stolen` - A boolean indicating whether this activity can be
    /// stolen or not.
    /// * `expects_events` - A boolean indicating whether this activity expects
    /// events or not. Setting this flag to true, when applicable,
    /// might increase performance.
    ///
    /// # Returns
    /// * `ActivityIdentifier` - The generated Activity Identifier for
    /// this Activity
    fn submit(
        &mut self,
        activity: Arc<Mutex<ActivityTrait>>,
        context: &Context,
        may_be_stolen: bool,
        expects_events: bool,
    ) -> ActivityIdentifier {
        self.thread_handler.as_mut().unwrap().submit(
            activity,
            context,
            may_be_stolen,
            expects_events,
        )
    }

    /// Perform a send operation with the event specified as argument
    ///
    /// # Arguments
    /// * `e` - Event to send
    fn send(&mut self, e: Box<Event>) {
        self.thread_handler.as_mut().unwrap().send(e);
    }

    /// Signal Constellation that it is done, perform a graceful shutdown of
    /// all threads and the thread_handler
    ///
    /// # Returns
    /// * `Result<bool, ConstellationError>` - Result type containing true if
    /// it could successfully shutdown, false otherwise.
    ///
    /// Upon error a ConstellationError is returned
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
                self.signal_thread_handler
                    .as_ref()
                    .unwrap()
                    .0
                    .send(true)
                    .expect("Failed to send signal to load balancer");

                let time = time::Duration::from_secs(100);
                if self.debug {
                    info!("Waiting for {}s for load balancer to shut down", 100);
                }
                if let Ok(r) = self
                    .signal_thread_handler
                    .as_ref()
                    .unwrap()
                    .1
                    .recv_timeout(time)
                {
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

    /// Retrieve an identifier for this Constellation instance
    ///
    /// # Returns
    /// * `ConstellationIdentifier` - Identifier for this Constellation instance
    fn identifier(&mut self) -> ConstellationIdentifier {
        self.const_id.clone()
    }

    fn is_master(&self) -> Result<bool, ConstellationError> {
        Ok(mpi_info::master(&self.universe))
    }

    fn nodes(&mut self) -> i32 {
        mpi_info::size(&self.universe)
    }
}

impl MultiThreadedConstellation {
    pub fn new(config: Box<ConstellationConfiguration>) -> MultiThreadedConstellation {
        let universe = mpi::initialize().unwrap();

        MultiThreadedConstellation {
            const_id: ConstellationIdentifier::new(&universe, Arc::new(Mutex::new(0)), -1),
            thread_handler: None,
            signal_thread_handler: None,
            universe,
            debug: config.debug,
            thread_count: config.number_of_threads,
            config,
        }
    }
}
