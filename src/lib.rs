#[macro_use]
extern crate mopa;
extern crate hashbrown;
extern crate objekt;
#[macro_use]
extern crate log;
extern crate simple_logger;

pub mod activity;
pub mod activity_identifier;
pub mod constellation;
pub mod constellation_config;
pub mod constellation_factory;
pub mod constellation_identifier;
pub mod context;
pub mod event;
pub mod implementation;
pub mod payload;
pub mod steal_strategy;
pub mod util;

pub use util::activities::single_event_collector::SingleEventCollector;
