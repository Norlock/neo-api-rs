use std::sync::{Mutex, MutexGuard};


pub trait FastLock<T> {
    fn fast_lock(&self) -> MutexGuard<'_, T>;
}

impl<T> FastLock<T> for Mutex<T> {
    fn fast_lock(&self) -> MutexGuard<'_, T> {
        self.lock().unwrap()
    }
}
