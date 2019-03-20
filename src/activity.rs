use super::constellation::ConstellationTrait;
use super::event::Event;

pub static FINISH: usize = 0;
pub static SUSPEND: usize = 1;

pub trait ActivityTrait: Sync + Send {
    fn cleanup(&mut self, constellation: &ConstellationTrait);
    fn initialize(&mut self, constellation: &ConstellationTrait) -> usize;
    fn process(&mut self, constellation: &ConstellationTrait, event: Event) -> usize;
}
