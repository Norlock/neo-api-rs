mod buffer;
mod callback;
mod neo_api;
mod neo_api_types;
mod popup;
mod window;
mod fuzzy;

use std::sync::{Mutex, MutexGuard};

pub use buffer::*;
pub use callback::*;
pub use neo_api::*;
pub use neo_api_types::*; 
pub use popup::*;
pub use window::*;
pub use fuzzy::*;

pub use mlua;

pub trait FastLock<T> {
    fn fast_lock(&self) -> MutexGuard<'_, T>;
}

impl<T> FastLock<T> for Mutex<T> {
    fn fast_lock(&self) -> MutexGuard<'_, T> {
        self.lock().unwrap()
    }
}
