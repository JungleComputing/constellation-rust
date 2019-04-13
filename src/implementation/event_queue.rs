///! Wrapper module for the Event HashMap, unique for each thread. This module
///! makes sure there can be multiple events sent to the same destination,
///! by extending the ordinary HashMap (from hashbrown) to hold a vector of
///! Events as value.

use crate::{Event, ActivityIdentifier};

use hashbrown::hash_map::Keys;
use hashbrown::HashMap;

/// EventQueue struct, for handling multiple Events per ActivityIdentifier
///
/// # Members
/// * `data` - The HashMap containing as key the ActivityIdentifiers
/// representing the destination activity as well as a vector of Events which
/// should go there.
pub struct EventQueue {
    data: HashMap<ActivityIdentifier, Vec<Box<Event>>>,
}

impl EventQueue {
    pub fn new() -> EventQueue {
        EventQueue {
            data: HashMap::new(),
        }
    }

    pub fn insert(&mut self, key: ActivityIdentifier, event: Box<Event>){
        self.data.entry(key).or_insert_with(Vec::new).push(event);
    }

    /// If there are multiple events, only one is returned. When the last one
    /// is returned, the entry is removed.
    pub fn remove(&mut self, key: ActivityIdentifier) -> Option<Box<Event>>{
        let mut event: Option<Box<Event>> = None;
        self.data.entry(key.clone()).and_modify(|e| event = e.pop());

        let empty = self.data.get(&key);

        if empty.is_some() && empty.unwrap().is_empty() {
            self.data.remove(&key);
        }
        event
    }

    pub fn contains_key(&mut self, key: &ActivityIdentifier) -> bool{
        self.data.contains_key(key)
    }

    pub fn is_empty(&self) -> bool {
        self.data.is_empty()
    }

    pub fn len(&self) -> usize {
        self.data.len()
    }

    pub fn keys(&self) -> Keys<ActivityIdentifier, Vec<Box<Event>>> {
        self.data.keys()
    }
}