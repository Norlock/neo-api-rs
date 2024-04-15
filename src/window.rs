#![allow(unused)]
use crate::{
    neo_api::NeoApi,
    neo_api_types::{OptValueType, WinCursor},
};
use mlua::prelude::{IntoLua, Lua, LuaResult};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Window(u32);

impl Window {
    pub const ZERO: Self = Self(0);

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

    pub fn set_cursor(&self, lua: &Lua, cursor: WinCursor) -> LuaResult<()> {
        NeoApi::win_set_cursor(lua, self.id(), cursor)
    }
}
