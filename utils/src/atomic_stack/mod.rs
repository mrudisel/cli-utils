
use std::sync::Mutex;

pub struct AtomicStack<T: Clone + Sized> {
    vec: Vec<T>,
    lock: Mutex<u32>,
}

impl<T: Clone + Sized> AtomicStack<T> {
    pub fn new() -> AtomicStack<T> {
        Self {
            vec: Vec::new(),
            lock: Mutex::new(0),
        }
    }

    pub fn with_capacity(cap: usize) -> AtomicStack<T> {
        Self {
            vec: Vec::with_capacity(cap),
            lock: Mutex::new(0),
        }
    }

    pub fn push(&mut self, item: T) -> bool {
        let _lock = match self.lock.lock() {
            Err(_) => return false,
            Ok(lock) => lock,
        };

        self.vec.push(item);
        true
    }

    pub fn pop(&mut self) -> Option<T> {
        let _lock = match self.lock.lock() {
            Err(_) => return None,
            Ok(lock) => lock,
        };

        self.vec.pop()
    }

    pub fn to_vec(&self) -> Option<Vec<T>> {
        let _lock = match self.lock.lock() {
            Err(_) => return None,
            Ok(lock) => lock,
        };

        Some(self.vec.to_vec())
    }
}

impl<T: Clone> From<&[T]> for AtomicStack<T> {
    fn from(s: &[T]) -> AtomicStack<T> {
        Self {
            vec: s.to_vec(),
            lock: Mutex::new(0),
        }
    }
}


impl<T: Clone> From<&mut [T]> for AtomicStack<T> {
    fn from(s: &mut [T]) -> AtomicStack<T> {
        Self {
            vec: s.to_vec(),
            lock: Mutex::new(0),
        }
    }
}
