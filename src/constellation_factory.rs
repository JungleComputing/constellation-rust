///! Use this struct to retrieve a ConstellationInstance, specify if you wish
///! to run single/multi-threaded or distributed using the Mode enum.

use crate::{ConstellationConfiguration, SingleThreadConstellation, MultiThreadedConstellation, ConstellationTrait};

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
            Box::from(SingleThreadConstellation::new(config))
        }
        Mode::MultiThreaded => {
            if config.number_of_threads == 1 && config.debug {
                info!("Only one thread specified for multithreaded constellation, returning single threaded instead");
                return Box::from(SingleThreadConstellation::new(config));
            }

            Box::from(MultiThreadedConstellation::new(config))
        }
        Mode::Distributed => unimplemented!(),
    }
}
