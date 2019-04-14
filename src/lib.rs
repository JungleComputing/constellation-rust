#[macro_use]
extern crate mopa;
extern crate hashbrown;
extern crate objekt;
#[macro_use]
extern crate log;
extern crate simple_logger;

pub mod constellation;
pub mod constellation_config;
pub mod constellation_factory;
pub mod context;
pub mod event;
pub mod implementation;
pub mod payload;
pub mod util;
pub mod activity;
pub mod steal_strategy;
pub mod error;

pub use util::activities::single_event_collector::SingleEventCollector;
pub use event::Event;
pub use context::{ContextVec, Context};
pub use payload::{PayloadTrait, PayloadTraitClone};
pub use activity_identifier::ActivityIdentifier;
pub use activity::ActivityTrait;
pub use constellation_factory::new_constellation;
pub use constellation_config::ConstellationConfiguration;
pub use constellation::ConstellationTrait;
pub use implementation::constellation_files::single_threaded_constellation::SingleThreadConstellation;
pub use implementation::constellation_files::multi_threaded_constellation::MultiThreadedConstellation;
pub use implementation::activity_identifier;
pub use steal_strategy::StealStrategy;
pub use error::ConstellationError;
