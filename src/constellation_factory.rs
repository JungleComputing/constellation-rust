use super::constellation::ConstellationTrait;
use super::constellation_config::ConstellationConfiguration;
use super::implementation::multi_threaded_constellation::multi_threaded_constellation;
use super::implementation::single_threaded_constellation::single_threaded_constellation;

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
            Box::from(single_threaded_constellation::SingleThreadConstellation::new(config))
        }
        Mode::MultiThreaded => {
            if config.number_of_threads == 1 && config.debug {
                info!("Only one thread specified for multithreaded constellation, returning single threaded instead");
                return Box::from(single_threaded_constellation::SingleThreadConstellation::new(config));
            }

            Box::from(multi_threaded_constellation::MultiThreadedConstellation::new(config))
        }
        Mode::Distributed => unimplemented!(),
    }
}
