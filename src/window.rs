#![allow(unused)]
use crate::{
    neo_api::NeoApi,
    neo_api_types::{OptValueType, WinCursor},
    NeoBuffer,
};

use mlua::prelude::{IntoLua, Lua, LuaFunction, LuaResult, LuaValue};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Default)]
pub struct NeoWindow(u32);

impl NeoWindow {
    pub const CURRENT: Self = Self(0);

    pub fn new(id: u32) -> Self {
        Self(id)
    }

    pub fn id(&self) -> u32 {
        self.0
    }

    pub fn get_current_win(lua: &Lua) -> LuaResult<NeoWindow> {
        let lfn: LuaFunction = lua.load("vim.api.nvim_get_current_win").eval()?;

        let buf_id = lfn.call(())?;

        Ok(NeoWindow::new(buf_id))
    }

    pub fn set_buf(&self, lua: &Lua, buf: &NeoBuffer) -> LuaResult<()> {
        let lfn: LuaFunction = lua.load("vim.api.nvim_win_set_buf").eval()?;

        lfn.call((self.id(), buf.id()))
    }

    /**
    Sets the value of an option. The behavior of this function matches that of
    |:set|: for global-local options, both the global and local value are set
    unless otherwise specified with {scope}.

    Parameters: ~
      • {name}   Option name
      • {value}  New option value
    */
    pub fn set_option_value<V: IntoLua>(
        &self,
        lua: &Lua,
        key: &str,
        value: V,
    ) -> LuaResult<()> {
        NeoApi::set_option_value(lua, key, value, OptValueType::Window(*self))
    }

    /**
    Sets the (1,0)-indexed cursor position in the window. |api-indexing| This
    scrolls the window even if it is not the current one.

    Parameters: ~
      • Window handle, or 0 for current window
      • WinCursor
    */
    pub fn set_cursor(&self, lua: &Lua, cursor: WinCursor) -> LuaResult<()> {
        let lfn: LuaFunction = lua.load("vim.api.nvim_win_set_cursor").eval()?;

        lfn.call((self.id(), cursor))
    }

    /**
    Gets the (1,0)-indexed, buffer-relative cursor position for a given window
    (different windows showing the same buffer have independent cursor
    positions). |api-indexing|

    Parameters: ~
      • {window}  Window handle, or 0 for current window

    See also: ~
      • |getcurpos()|
    */
    pub fn get_cursor(&self, lua: &Lua) -> LuaResult<WinCursor> {
        let lfn: LuaFunction = lua.load("vim.api.nvim_win_get_cursor").eval()?;

        lfn.call(self.id())
    }

    /// Adds the namespace scope to the window.
    pub fn add_ns(&self, lua: &Lua, ns_id: u32) -> LuaResult<()> {
        let lfn: LuaFunction = lua.load("vim.api.nvim_win_add_ns").eval()?;

        lfn.call((self.id(), ns_id))
    }

    pub fn call(&self, lua: &Lua, cb: LuaFunction) -> LuaResult<()> {
        let lfn: LuaFunction = lua.load("vim.api.nvim_win_call").eval()?;

        lfn.call((self.id(), cb))
    }

    /**
    nvim_win_close({window}, {force})                           *nvim_win_close()*
    Closes the window (like |:close| with a |window-ID|).

    Attributes: ~
        not allowed when |textlock| is active
    */
    pub fn close(&self, lua: &Lua, force: bool) -> LuaResult<()> {
        let lfn: LuaFunction = lua.load("vim.api.nvim_win_close").eval()?;

        lfn.call((self.id(), force))
    }
}
