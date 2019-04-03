use super::constellation::ConstellationTrait;
use super::constellation_config::ConstellationConfiguration;
use super::implementation::single_constellation::single_threaded_constellation;

/// Use to specify which constellation instance to create
pub enum Mode {
    SingleThreaded,
    MultiThreaded,
    Distributed,
}

pub fn new_constellation(
    mode: Mode,
    config: Box<ConstellationConfiguration>,
) -> Box<dyn ConstellationTrait> {
    match mode {
        Mode::SingleThreaded => {
            return Box::from(single_threaded_constellation::SingleThreadConstellation::new(config));
        }
        Mode::MultiThreaded => unimplemented!(),
        Mode::Distributed => unimplemented!(),
    }
}
