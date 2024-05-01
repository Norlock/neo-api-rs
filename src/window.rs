#![allow(unused)]
use crate::{
    neo_api::NeoApi,
    neo_api_types::{OptValueType, WinCursor},
};
use mlua::prelude::{IntoLua, Lua, LuaResult, LuaFunction};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct NeoWindow(u32);

impl NeoWindow {
    pub const CURRENT: Self = Self(0);

    pub fn new(id: u32) -> Self {
        Self(id)
    }

    pub fn id(&self) -> u32 {
        self.0
    }

    /**
    Sets the value of an option. The behavior of this function matches that of
    |:set|: for global-local options, both the global and local value are set
    unless otherwise specified with {scope}.

    Parameters: ~
      • {name}   Option name
      • {value}  New option value
    */
    pub fn set_option_value<'a, V: IntoLua<'a>>(
        &self,
        lua: &'a Lua,
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

        lfn.call::<_, ()>((self.id(), cursor))
    }


    /**
    Gets the (1,0)-indexed, buffer-relative cursor position for a given window
    (different windows showing the same buffer have independent cursor
    positions). |api-indexing|

    Parameters: ~
      • {window}  Window handle, or 0 for current window

    Return: ~
        (row, col) tuple

    See also: ~
      • |getcurpos()|
    */
    pub fn get_cursor(&self, lua: &Lua) -> LuaResult<WinCursor> {
        let lfn: LuaFunction = lua.load("vim.api.nvim_win_get_cursor").eval()?;

        lfn.call::<_, WinCursor>((self.id()))
    }

    /**
    nvim_win_close({window}, {force})                           *nvim_win_close()*
    Closes the window (like |:close| with a |window-ID|).

    Attributes: ~
        not allowed when |textlock| is active

    Parameters: ~
      • {window}  Window handle, or 0 for current window
      • {force}   Behave like `:close!` The last window of a buffer with
                  unwritten changes can be closed. The buffer will become
                  hidden, even if 'hidden' is not set.
    */
    pub fn close(&self, lua: &Lua, force: bool) -> LuaResult<()> {
        let lfn: LuaFunction = lua.load("vim.api.nvim_win_close").eval()?;

        lfn.call::<_, ()>((self.id(), force))
    }
}
