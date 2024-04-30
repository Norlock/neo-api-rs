use mlua::Lua;
use mlua::prelude::LuaTable;

pub trait CreateTable {
    fn create_table<'a>(&self, lua: &'a Lua) -> LuaTable<'a>;
}
