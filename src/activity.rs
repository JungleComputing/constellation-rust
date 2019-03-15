use super::activity_identifier::ActivityIdentifierImpl;
use super::constellation::ConstellationTrait;
use super::event::Event;

pub static FINISH: usize = 0;
pub static SUSPEND: usize = 1;

struct ActivityWrapper<'a> {
    id: &'a ActivityIdentifierImpl,
    may_be_stolen: bool,
    expects_events: bool,
}

impl<'a> ActivityWrapper<'a> {
    fn identifier(self) -> &'a ActivityIdentifierImpl{
        self.id
    }
}

pub trait ActivityTrait {
    fn cleanup(&self, constellation: &ConstellationTrait);
    fn initialize(&self, constellation: &ConstellationTrait);
    fn process(&self, constellation: &ConstellationTrait, event: Event);
}