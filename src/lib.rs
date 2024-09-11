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
mod utils;

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
pub use utils::*;
pub use diffuser::*;

pub use mlua;
pub use tokio;
pub use async_trait;

use tokio::runtime::Runtime;
use std::sync::LazyLock;

/// Tokio runtime multithreaded
pub static RTM: LazyLock<Runtime> = LazyLock::new(|| tokio::runtime::Runtime::new().unwrap());
