use std::sync::{Mutex, MutexGuard};

use mlua::IntoLua;

pub trait FastLock<T> {
    fn fast_lock(&self) -> MutexGuard<'_, T>;
}

impl<T> FastLock<T> for Mutex<T> {
    fn fast_lock(&self) -> MutexGuard<'_, T> {
        self.lock().unwrap()
    }
}

pub struct Phone {
    pub number: String,
    pub mobile: Option<String>,
}

impl<'a> IntoLua<'a> for Phone {
    fn into_lua(
        self,
        lua: &'a mlua::prelude::Lua,
    ) -> mlua::prelude::LuaResult<mlua::prelude::LuaValue<'a>> {
        let out = lua.create_table()?;
        out.set("number", self.number)?;
        out.set("mobile", self.mobile)?;

        Ok(mlua::Value::Table(out))
    }
}

pub struct Test {
    pub person: Vec<Phone>,
}

impl<'a> IntoLua<'a> for Test {
    fn into_lua(
        self,
        lua: &'a mlua::prelude::Lua,
    ) -> mlua::prelude::LuaResult<mlua::prelude::LuaValue<'a>> {
        let out = lua.create_table()?;
        out.set("person", self.person)?;

        Ok(mlua::Value::Table(out))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test() -> mlua::Result<()> {
        let phone = Phone {
            number: "01214".to_string(),
            mobile: Some("13413".to_string())
        };

        let a = Test {
            person: vec![phone] 
        };

        let lua = mlua::Lua::new();
        let table = a.into_lua(&lua)?;

        assert!(table.is_table());

        Ok(())
    }
}
