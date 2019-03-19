use super::constellation::ConstellationTrait;
use super::event::Event;

pub static FINISH: usize = 0;
pub static SUSPEND: usize = 1;

pub trait ActivityTrait: Sync + Send {
    fn cleanup(&self, constellation: &ConstellationTrait);
    fn initialize(&self, constellation: &ConstellationTrait) -> usize;
    fn process(&self, constellation: &ConstellationTrait, event: Event) -> usize;
}
