mod bridge;
mod buffer;
mod diffuser;
mod search;
mod neo_api;
mod neo_api_types;
mod popup;
mod theme;
mod traits;
mod web_devicons;
mod window;
mod debug;
mod database;

pub use bridge::*;
pub use buffer::*;
pub use search::*;
pub use neo_api::*;
pub use neo_api_types::*;
pub use popup::*;
pub use theme::*;
pub use traits::*;
pub use window::*;
pub use debug::*;
pub use database::*;
pub use diffuser::ExecuteTask;

pub use mlua;
pub use tokio;

use once_cell::sync::Lazy;
use tokio::runtime::Runtime;

/// Tokio runtime multithreaded
pub static RTM: Lazy<Runtime> = Lazy::new(|| tokio::runtime::Runtime::new().unwrap());
