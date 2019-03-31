use crate::constellation_config::ConstellationConfiguration;
use crate::constellation::ConstellationTrait;

use super::single_threaded_constellation;

pub enum Mode{
    SingleThreaded,
    MultiThreaded,
    Distributed,
}

pub fn new_constellation(mode: Mode, config: Box<ConstellationConfiguration>) -> Box<dyn ConstellationTrait> {
    match mode {
        Mode::SingleThreaded => {
            return Box::from(single_threaded_constellation::SingleThreadConstellation::new(config));
        },
        Mode::MultiThreaded => {
            unimplemented!()
        },
        Mode::Distributed => {
            unimplemented!()
        }
    }
}

