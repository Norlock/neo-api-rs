mod buffer;
mod callback;
mod neo_api;
mod neo_api_types;
mod popup;
mod traits;
mod window;

pub use buffer::*;
pub use callback::*;
pub use neo_api::*;
pub use neo_api_types::*; 
use once_cell::sync::Lazy;
pub use popup::*;
use tokio::runtime::{self, Runtime};
pub use traits::*;
pub use window::*;

pub use mlua;

static RUNTIME: Lazy<Runtime> = Lazy::new(|| {
    runtime::Builder::new_multi_thread()
        .enable_io()
        .build()
        .unwrap()
});

