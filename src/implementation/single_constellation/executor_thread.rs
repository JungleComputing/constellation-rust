extern crate crossbeam;

use std::sync::{Arc, Mutex};
use std::time;
use std::thread;

use crate::event::Event;
use crate::constellation::ConstellationTrait;
use super::super::activity_wrapper::ActivityWrapperTrait;

use crossbeam::deque;
use crossbeam::deque::Steal;
use crossbeam::{Sender, Receiver};

/// Executor thread, runs in a separate thread and is in charge of executing
/// activities. It will periodically check for work in the Constellation
/// instance using it's shared queues.
///
/// * `sender` - Sender channel to send data to the Constellation instance.
/// * `receiver` - Receiver channel to receive data from the Constellation instance.
/// * `work_queue` - Shared queue with Constellation instance, used to grab
/// work when available.
/// * `event_queue` - Shared queue for events containing data, executor will
/// check this queue whenever if is expecting events
/// * `constellation` - A reference to the InnerConstellation instance, required
/// by the functions in the activities executed
pub struct ExecutorThread {
    sender: Sender<i32>,
    receiver: Receiver<i32>,
    work_queue: Arc<Mutex<deque::Injector<Box<dyn ActivityWrapperTrait>>>>,
    event_queue: Arc<Mutex<deque::Injector<Box<Event>>>>,
    local: deque::Worker<Box<dyn ActivityWrapperTrait>>,
    constellation: Arc<Mutex<Box<dyn ConstellationTrait>>>,
}

impl ExecutorThread {
    /// Create a new ExecutorThread
    ///
    /// * `ExecutorThread` - New executor thread
    pub fn new(
        sender: Sender<i32>,
        receiver: Receiver<i32>,
        work_queue: Arc<Mutex<deque::Injector<Box<dyn ActivityWrapperTrait>>>>,
        event_queue: Arc<Mutex<deque::Injector<Box<Event>>>>,
        constellation: Arc<Mutex<Box<dyn ConstellationTrait>>>,
    ) -> ExecutorThread{
        ExecutorThread {
            sender,
            receiver,
            work_queue,
            event_queue,
            local: deque::Worker::new_fifo(),
            constellation,
        }
    }

    /// Tries to steal a batch of work from the shared work_queue. If there is
    /// work, it will return one of the stolen jobs, which is to be
    /// executed immediately.
    ///
    /// # Returns
    /// * `Option<Box<dyn ActivityWrapperTrait>>` - If there is work, it will
    /// pop one job from the local queue and return that wrapped in Some(..)
    fn check_for_work(&mut self) -> Option<Box<dyn ActivityWrapperTrait>> {
        // Steal work from shared activity queue, if available
        if let Steal::Success(activity) = self.work_queue.lock().unwrap()
            .steal_batch_and_pop(&self.local) {
            return Some(activity);
        }

        None
    }

    /// Process a stolen activity, this is the main function of executing a
    /// activity.
    ///
    /// ADD MORE WHEN FURTHER
    ///
    /// # Arguments
    /// * `activity` - A boxed activity to perform work on.
    fn process_work(&mut self, mut activity: Box<dyn ActivityWrapperTrait>) {
        self.initialize(activity);
    }

    /// Call initialize on the activity
    fn initialize(&mut self, mut activity: Box<dyn ActivityWrapperTrait>){
        activity.initialize(self.constellation.clone());
    }

    /// This will startup the thread, periodically check for work forever or
    /// if shut down from InnerConstellation/SingleThreadedConstellation.
    pub fn run(&mut self) {
        let time = time::Duration::from_secs(1);
        thread::sleep(time);

        for _ in 0..20 {
            let mut work = self.check_for_work();
            match work {
                Some(x) => self.process_work(x),
                None => (),
            }
        }
    }
}