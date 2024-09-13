mod bridge;
mod buffer;
mod database;
mod debug;
mod neo_api;
mod neo_api_types;
mod popup;
mod search;
mod theme;
mod traits;
mod utils;
mod web_devicons;
mod window;

pub use bridge::*;
pub use buffer::*;
pub use database::*;
pub use debug::*;
pub use neo_api::*;
pub use neo_api_types::*;
pub use popup::*;
pub use search::*;
pub use theme::*;
pub use traits::*;
pub use utils::*;
pub use window::*;

pub use async_trait;
pub use mlua;
pub use tokio;

use std::sync::LazyLock;
use tokio::runtime::Runtime;

/// Tokio runtime multithreaded
pub static RTM: LazyLock<Runtime> = LazyLock::new(|| tokio::runtime::Runtime::new().unwrap());
