#[macro_use]
extern crate mopa;
extern crate hashbrown;
extern crate objekt;

pub mod activity;
pub mod activity_identifier;
pub mod constellation;
pub mod constellation_config;
pub mod constellation_factory;
pub mod constellation_identifier;
pub mod constellation_properties;
pub mod context;
pub mod event;
pub mod implementation;
pub mod message;
pub mod steal_strategy;
pub mod util;

pub use implementation::single_constellation::single_threaded_constellation;

// Utils
pub use util::activities::single_event_collector::SingleEventCollector;
