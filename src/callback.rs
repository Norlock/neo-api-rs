use mlua::Lua;
use once_cell::sync::Lazy;
use tokio::sync::{Mutex, MutexGuard};

use crate::prelude::AutoCmdCbEvent;

static CB_CONTAINER: Lazy<Mutex<CallBackState>> = Lazy::new(|| {
    let cb_state = CallBackState::default();

    Mutex::new(cb_state)
});

/// Because autocmd callbacks are invoked before returning the NeoApi function calls
/// It can deadlock your app, this makes sure a queue is added which can be called
/// at the end of any module function implementation
pub struct CbContainer;

pub type CbFunction = Box<dyn Fn(&Lua, AutoCmdCbEvent) -> ()>;

impl CbContainer {
    pub async fn add_to_queue(func: CbFunction, ev: AutoCmdCbEvent) {
        let mut cb_container = CB_CONTAINER.lock().await;
        cb_container.queue.push(CbArgs { ev, func });
    }

    pub async fn exec<T>(lua: &Lua) {
        let mut cb_container = CB_CONTAINER.lock().await;
        while !cb_container.queue.is_empty() {
            let item = cb_container.queue.remove(0);
            (item.func)(lua, item.ev);
        }
    }

    /// If your global state has an active lock 
    /// This function will drop as a reminder
    pub async fn exec_drop_lock<T>(guard: MutexGuard<'_, T>, lua: &Lua) {
        drop(guard);
        let mut cb_container = CB_CONTAINER.lock().await;
        while !cb_container.queue.is_empty() {
            let item = cb_container.queue.remove(0);
            (item.func)(lua, item.ev);
        }
    }
}

pub struct CbArgs {
    pub func: CbFunction,
    pub ev: AutoCmdCbEvent,
}

pub struct CallBackState {
    pub queue: Vec<CbArgs>,
}

unsafe impl Send for CallBackState {}
unsafe impl Sync for CallBackState {}

impl Default for CallBackState {
    fn default() -> Self {
        Self { queue: vec![] }
    }
}