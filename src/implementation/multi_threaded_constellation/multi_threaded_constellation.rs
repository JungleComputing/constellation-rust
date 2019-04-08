use std::sync::{Arc, Mutex};

use crate::activity::ActivityTrait;
use crate::activity_identifier::ActivityIdentifier;
use crate::constellation::ConstellationTrait;
use crate::constellation_config::ConstellationConfiguration;
use crate::constellation_identifier::ConstellationIdentifier;
use crate::context::Context;
use crate::event::Event;
use crate::implementation::error::ConstellationError;
use crate::implementation::single_threaded_constellation::single_threaded_constellation::SingleThreadConstellation;
use hashbrown::HashMap;
use crate::implementation::activity_wrapper::ActivityWrapperTrait;
use super::super::mpi::environment::Universe;
use crate::implementation::communication::mpi_info;

use crossbeam::deque;
use crate::implementation::single_threaded_constellation::inner_constellation::InnerConstellation;


pub struct MultiThreadedConstellation {
    inner_constellation_vec: Vec<Arc<Mutex<Box<dyn ConstellationTrait>>>>,
    local_work: HashMap<ActivityIdentifier, Box<dyn ActivityWrapperTrait>>,
    universe: Universe,
    debug: bool,
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
            for x in 0..self.inner_constellation_vec.len() {
                let mut work_queue: Arc<Mutex<deque::Injector<Box<dyn ActivityWrapperTrait>>>>;
                let mut event_queue: Arc<Mutex<deque::Injector<Box<Event>>>>;

                if let Some(inner) = self.inner_constellation_vec[x]
                    .lock()
                    .unwrap()
                    .downcast_ref::<InnerConstellation>()
                {
                    work_queue = inner.work_queue.clone();
                    event_queue = inner.event_queue.clone();
                } else {
                    panic!("Something went wrong when cloning the work and event queue")
                };

                self.inner_constellation_vec[x]
                    .lock()
                    .unwrap()
                    .downcast_mut::<InnerConstellation>()
                    .unwrap()
                    .activate_inner(self.inner_constellation_vec[x].clone(), work_queue, event_queue);
            }

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
        self.inner_constellation_vec[0].lock().unwrap().submit(activity, context, may_be_stolen, expects_events)
    }

    fn send(&mut self, e: Box<Event>) {
        self.inner_constellation_vec[0].lock().unwrap().send(e)
    }

    fn done(&mut self) -> Result<bool, ConstellationError> {
        if self.debug {
            info!("Attempting to shut down Constellation gracefully");
        }

        for x in 0..self.inner_constellation_vec.len() {
            if !self.inner_constellation_vec[x].lock().expect(
                "Could not get lock on constellation instance"
            ).done().unwrap() {
                warn!("Thread {} was not done, aborting shutdown", x);
                return Ok(false);
            }
        }

        Ok(true)
    }

    fn identifier(&mut self) -> ConstellationIdentifier {
        unimplemented!()
    }

    fn is_master(&mut self) -> Result<bool, ConstellationError> {
        Ok(mpi_info::master(&self.universe))
    }

    fn nodes(&mut self) -> i32 {
        mpi_info::size(&self.universe)
    }

    fn set_parent(&mut self, parent: Arc<Mutex<Box<dyn ConstellationTrait>>>){
        unimplemented!();
    }
}

impl MultiThreadedConstellation {

    /// Create
    pub fn new(config: Box<ConstellationConfiguration>) -> MultiThreadedConstellation {
        // Start all single threaded constellation instances, i.e.
        // InnerConstellation

        let mut const_vec: Vec<Arc<Mutex<Box<dyn ConstellationTrait>>>> = Vec::new();
        let universe = mpi::initialize().unwrap();
        if mpi_info::master(&universe) {
            // Only start up inner constellation instances for the master
            for x in 0..config.number_of_threads {
                const_vec.push(SingleThreadConstellation::create_only_inner(config.clone(), &universe, x));
            }
        }

        let multi = MultiThreadedConstellation {
            inner_constellation_vec: const_vec,
            local_work: HashMap::new(),
            universe,
            debug: false
        };

        for inner in &multi.inner_constellation_vec {
            let inner_clone = inner.clone();
            inner.lock().unwrap().set_parent(inner_clone);
        }

        multi
    }
}
