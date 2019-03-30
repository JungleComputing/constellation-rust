use crate::constellation::ConstellationTrait;
use crate::constellation_identifier::ConstellationIdentifier;
use std::sync::Arc;
use std::sync::Mutex;
use crate::activity::ActivityTrait;
use crate::context::Context;
use crate::implementation::error::ConstellationError;
use crate::event::Event;
use crate::activity_identifier::ActivityIdentifier;
use std::sync::MutexGuard;
use crate::implementation::activity_wrapper::ActivityWrapper;
use crate::implementation::activity_wrapper::ActivityWrapperTrait;
use super::mpi::environment::Universe;
use super::mpi::topology::SystemCommunicator;
use super::mpi::topology::Communicator;
use crossbeam::deque;


/// Inner single threaded Constellation instance, shared with the application.
/// The parent will be the multi threaded constellation handler
pub struct InnerConstellation {
    identifier: Arc<Mutex<ConstellationIdentifier>>,
    universe: Universe,
    pub work_queue: Arc<Mutex<deque::Injector<Box<dyn ActivityWrapperTrait>>>>,
    pub event_queue: Arc<Mutex<deque::Injector<Box<Event>>>>,
    pub parent: Option<Arc<Mutex<dyn ConstellationTrait>>>,
}

impl ConstellationTrait for InnerConstellation {
    fn activate(&mut self) -> Result<bool, ConstellationError> {
        panic!("This function should never be called from inside inner class");
    }

    fn submit(&mut self, activity: &Arc<Mutex<ActivityTrait>>,
              context: &Context,
              may_be_stolen: bool,
              expects_events: bool) -> ActivityIdentifier {

        let activity_wrapper =
            ActivityWrapper::new(self.identifier.clone(),
                                 activity,
                                 context,
                                 may_be_stolen,
                                 expects_events);

        let activity_id = activity_wrapper.activity_identifier();

        // Insert ActivityWrapper in injector_queue
        self.work_queue
            .lock()
            .expect("Could not get lock on injector_queue, failed to push activity")
            .push(activity_wrapper);

        activity_id
    }

    fn send(&mut self, e: Event) {
        unimplemented!();
    }

    fn done(&mut self) -> Result<bool, ConstellationError> {
        unimplemented!();
    }

    fn identifier(&mut self) -> ConstellationIdentifier {
        self.identifier.lock().expect(
            "Could not get lock on ConstellationIdentifier"
        ).clone()
    }

    fn is_master(&mut self) -> Result<bool, ConstellationError> {
        if self.rank() == 0 {
            Ok(true)
        } else {
            Ok(false)
        }
    }

    fn nodes(&mut self) -> i32 {
        self.world().size()
    }

    fn generate_identifier(&mut self) -> ConstellationIdentifier {
        // Check if there is a multithreaded Constellation running
        if self.parent.is_none() {
            // Has no parent, this is the top-level instance of Constellation
            return ConstellationIdentifier::new(&self.universe);
        }

        // Call parent method ConstellationIdentifier
        self.parent.as_mut().expect(
            "No parent available"
        ).lock().unwrap().generate_identifier()
    }
}

impl InnerConstellation {
    pub fn new(work_queue: Arc<Mutex<deque::Injector<Box<dyn ActivityWrapperTrait>>>>,
               event_queue: Arc<Mutex<deque::Injector<Box<Event>>>>) -> InnerConstellation {
        let mut id = Arc::new(Mutex::new(ConstellationIdentifier::new_empty()));
        let mut new_const = InnerConstellation {
            identifier: id,
            universe: mpi::initialize().unwrap(),
            work_queue,
            event_queue,
            parent: None
        };
        new_const.identifier = Arc::new(Mutex::new(new_const.generate_identifier()));

        new_const

    }

    fn rank(&self) -> i32 {
        self.world().rank()
    }

    fn world(&self) -> SystemCommunicator {
        self.universe.world()
    }

    pub fn set_parent(&mut self, parent: Arc<Mutex<dyn ConstellationTrait>>) {
        self.parent = Some(parent.clone());
    }
}