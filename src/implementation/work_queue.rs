use std::collections::VecDeque;

pub struct WorkQueue<T> {
    //TODO Implement WorkQueue with RefMut
    queue: VecDeque<T>,
}

impl<T> WorkQueue<T> {
    pub fn push_front(&mut self, data: T) {
        self.queue.push_front(data);
    }
    pub fn push_back(&mut self, data: T) {
        self.queue.push_back(data);
    }
    pub fn pop_front(&mut self) {
        self.queue.pop_front();
    }
    pub fn pop_back(&mut self) {
        self.queue.pop_back();
    }

    pub fn new() -> WorkQueue<T> {
        WorkQueue {
            queue: VecDeque::new(),
        }
    }
}
