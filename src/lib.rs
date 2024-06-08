mod bridge;
mod buffer;
mod diffuser;
mod fuzzy;
mod neo_api;
mod neo_api_types;
mod popup;
mod theme;
mod traits;
mod web_devicons;
mod window;

pub use bridge::*;
pub use buffer::*;
pub use fuzzy::*;
pub use neo_api::*;
pub use neo_api_types::*;
pub use popup::*;
pub use theme::*;
pub use traits::*;
pub use window::*;

pub use mlua;
pub use tokio;

use once_cell::sync::Lazy;
use tokio::runtime::Runtime;

/// Tokio runtime multithreaded
pub static RTM: Lazy<Runtime> = Lazy::new(|| tokio::runtime::Runtime::new().unwrap());
