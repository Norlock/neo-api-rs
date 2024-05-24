use mlua::prelude::{LuaError, LuaResult};
use once_cell::sync::Lazy;
use std::{any::Any, collections::HashMap};
use tokio::sync::RwLock;

static LUA_BRIDGE: Lazy<RwLock<NeoBridge>> = Lazy::new(|| RwLock::new(NeoBridge(HashMap::new())));

/// This can be used to store small data for inside lua async functions, 
/// where you can't move the data inside the closure.
pub struct NeoBridge(HashMap<String, Box<dyn Any>>);

unsafe impl Send for NeoBridge {}
unsafe impl Sync for NeoBridge {}

impl NeoBridge {
    pub async fn insert(key: &str, cb: Box<dyn Any>) {
        let mut bridge = LUA_BRIDGE.write().await;
        bridge.0.insert(key.to_string(), cb);
    }

    /// Will clone the stored data.
    pub async fn clone<T: 'static + Clone>(key: &str) -> LuaResult<T> {
        let bridge = LUA_BRIDGE.read().await;

        if let Some(cb) = bridge.0.get(key) {
            if let Some(downcast) = cb.downcast_ref::<T>() {
                return Ok(downcast.clone());
            } else {
                return Err(LuaError::external("Can't downcast to the expected type"));
            }
        }

        Err(LuaError::external("Item with key doesn't exist"))
    }

    /// Will consume the stored data.
    pub async fn consume<T: 'static + Clone>(key: &str) -> LuaResult<T> {
        let bridge = LUA_BRIDGE.read().await;

        if let Some(cb) = bridge.0.get(key) {
            if let Some(downcast) = cb.downcast_ref::<T>() {
                return Ok(downcast.clone());
            } else {
                return Err(LuaError::external("Can't downcast to the expected type"));
            }
        }

        Err(LuaError::external("Item with key doesn't exist"))
    }
}
