use crate::AutoCmdCbEvent;
use mlua::Lua;
use std::sync::OnceLock;
use std::sync::Mutex;
use mlua::prelude::LuaResult;

pub const fn create_callback_container<T>() -> OnceLock<Mutex<CallBackQueue<T>>> {
    OnceLock::new()
}

pub trait InitCBQueue<T> {
    fn init(&self);
    fn push(&self, func: CbFunction<T>, ev: AutoCmdCbEvent) -> LuaResult<()>;
    fn exec(&self, state: &mut T, lua: &Lua) -> LuaResult<()>;
}

impl<T> InitCBQueue<T> for OnceLock<Mutex<CallBackQueue<T>>> {
    fn init(&self) {
        let _ = self.set(Mutex::new(CallBackQueue::default()));
    }

    fn push(&self, func: CbFunction<T>, ev: AutoCmdCbEvent) -> LuaResult<()> {
        let mut queue = self.get().unwrap().lock().unwrap();
        queue.push(func, ev);

        Ok(())
    }

    fn exec(&self, state: &mut T, lua: &Lua) -> LuaResult<()> {
        let mut queue = self.get().unwrap().lock().unwrap();
        queue.exec(state, lua);

        Ok(())
    }
}

pub type CbFunction<T> = Box<dyn Fn(&Lua, &mut T, AutoCmdCbEvent)>;

pub struct CbArgs<T> {
    pub func: CbFunction<T>,
    pub ev: AutoCmdCbEvent,
}

/// Because autocmd callbacks are invoked before returning the NeoApi function calls
/// It can deadlock your app, this makes sure a queue is added which can be called
/// at the end of any module function implementation
pub struct CallBackQueue<T>(Vec<CbArgs<T>>);

impl<T> CallBackQueue<T> {
    fn push(&mut self, func: CbFunction<T>, ev: AutoCmdCbEvent) {
        self.0.push(CbArgs { ev, func });
    }

    fn exec(&mut self, state: &mut T, lua: &Lua) {
        while !self.0.is_empty() {
            let item = self.0.remove(0);
            // Execute the callback
            (item.func)(lua, state, item.ev);
        }
    }
}

unsafe impl<T> Send for CallBackQueue<T> {}
unsafe impl<T> Sync for CallBackQueue<T> {}

impl<T> Default for CallBackQueue<T> {
    fn default() -> Self {
        Self(vec![])
    }
}
