use std::collections::VecDeque;
use std::sync::mpsc::Sender;
use std::sync::mpsc::Receiver;

#[derive(Clone, Debug, PartialEq)]
pub enum Direction {
    Left,
    Right,
}

///´Source´'s are sources for some type T. Taking from a source returns an optional.
/// While a ´Source´ has things it should return Some(T).
/// If the ´Source´ permanently runs out of things it should return None signaling to
/// the user of the source should move on to do other things.
pub trait Source<T> {
    fn take(&mut self) -> Option<T>;
}

impl<T> Source<T> for Vec<T> {
    fn take(&mut self) -> Option<T> {
        if self.is_empty() {
            None
        } else {
            Some(self.remove(0))
        }
    }
}

impl<T> Source<T> for VecDeque<T> {
    fn take(&mut self) -> Option<T> {
        self.pop_front()
    }
}

impl<T> Source<T> for Receiver<T> {
    fn take(&mut self) -> Option<T> {
        self.recv().ok()
    }
}

/// Sinks are things take take in some type T.
pub trait Sink<T> {
    fn put(&mut self, thing: T);
}

impl<T> Sink<T> for Vec<T> {
    fn put(&mut self, thing: T) {
        self.push(thing);
    }
}

impl<T> Sink<T> for VecDeque<T> {
    fn put(&mut self, thing: T) {
        self.push_back(thing);
    }
}

impl<T> Sink<T> for Sender<T> {
    fn put(&mut self, thing: T) {
        self.send(thing).unwrap();
    }
}
