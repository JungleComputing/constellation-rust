use super::constellation::ConstellationTrait;
use super::event::Event;
use std::sync::Mutex;
use std::sync::Arc;

pub static FINISH: usize = 0;
pub static SUSPEND: usize = 1;

pub trait ActivityTrait: Sync + Send {
    fn cleanup(&mut self,  constellation: Arc<Mutex<Box<dyn ConstellationTrait>>>);
    fn initialize(&mut self, constellation: Arc<Mutex<Box<dyn ConstellationTrait>>>) -> usize;
    fn process(&mut self, constellation: Arc<Mutex<Box<dyn ConstellationTrait>>>, event: Event) -> usize;
}
