mod buffer;
mod fuzzy;
mod neo_api;
mod neo_api_types;
mod popup;
mod theme;
mod traits;
mod window;

pub use buffer::*;
pub use fuzzy::*;
pub use neo_api::*;
pub use neo_api_types::*;
use once_cell::sync::Lazy;
pub use popup::*;
pub use theme::*;
use tokio::runtime::Runtime;
pub use traits::*;
pub use window::*;

pub use mlua;
pub use tokio;

/// Tokio runtime multithreaded
pub static RTM: Lazy<Runtime> = Lazy::new(|| tokio::runtime::Runtime::new().unwrap());
