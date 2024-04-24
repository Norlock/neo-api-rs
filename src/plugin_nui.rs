#![allow(unused)]
use std::{
    collections::HashMap,
    ops::{Deref, DerefMut},
    sync::Arc,
};

use mlua::{
    prelude::{LuaFunction, LuaResult, LuaSerializeOptions, LuaString, LuaTable, LuaValue, LuaError},
    IntoLua, Lua, LuaSerdeExt,
};
use serde::Serialize;
use tokio::sync::Mutex;

use crate::prelude::{AutoCmdEvent, CbContainer, KeymapOpts, Mode, NeoApi};

#[derive(Debug, Default, Serialize, Clone, Copy, PartialEq, Eq)]
pub enum NuiRelative {
    #[default]
    Win,
    Cursor,
    Editor,
}

#[derive(Debug, Default, Serialize, Clone, Copy, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum NuiBorderStyle {
    #[default]
    None,
    Double,
    Rounded,
    Shadow,
    Single,
    Solid,
}

#[derive(Debug, Default, Serialize, Clone, Copy, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum NuiAlign {
    #[default]
    Center,
    Left,
    Right,
}

#[derive(Debug, Clone, Copy)]
pub enum NuiSize {
    Fixed(u16),
    Percentage(u16),
}

impl NuiSize {
    pub fn to_value<'lua>(&'lua self, lua: &'lua Lua) -> LuaValue<'lua> {
        match self {
            Self::Fixed(val) => LuaValue::Integer(*val as i64),
            Self::Percentage(val) => {
                LuaValue::String(lua.create_string(format!("{val}%")).unwrap())
            }
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub enum NuiDimension {
    XorY(NuiSize, NuiSize),
    XandY(NuiSize),
}

#[derive(Debug, Serialize, Clone)]
pub struct NuiBorderText {
    pub top: Option<String>,
    pub top_align: NuiAlign,
    pub bottom: Option<String>,
    pub bottom_align: NuiAlign,
}

#[derive(Debug, Default, Serialize, Clone, Copy)]
pub struct NuiBorderPadding {
    pub top: Option<u16>,
    pub left: Option<u16>,
    pub right: Option<u16>,
    pub bottom: Option<u16>,
}

#[derive(Debug, Serialize, Clone)]
pub struct NuiBorder {
    pub padding: Option<NuiBorderPadding>,
    pub style: Option<NuiBorderStyle>,
    pub text: Option<NuiBorderText>,
}

#[derive(Debug, Serialize, Clone)]
pub struct NuiBufOptions {
    pub modifiable: bool,
    pub readonly: bool,
}

#[derive(Debug, Serialize, Clone)]
pub struct NuiWinOptions {
    pub winblend: u16,
    pub winhighlight: String,
}

#[derive(Debug, Clone)]
pub struct NuiPopupOpts {
    pub position: NuiDimension,
    pub size: NuiDimension,
    pub enter: Option<bool>,
    pub focusable: Option<bool>,
    pub zindex: Option<u16>,
    pub relative: Option<NuiRelative>,
    pub border: Option<NuiBorder>,
    pub buf_options: Option<NuiBufOptions>,
    pub win_options: Option<NuiWinOptions>,
}

impl<'lua> IntoLua<'lua> for NuiPopupOpts {
    fn into_lua(self, lua: &'lua mlua::Lua) -> LuaResult<LuaValue<'lua>> {
        let mut ser_opts = LuaSerializeOptions::new();
        ser_opts.serialize_none_to_null = false;
        ser_opts.serialize_unit_to_null = false;

        let out = lua.create_table()?;

        let size = lua.create_table()?;

        match self.size {
            NuiDimension::XorY(x, y) => {
                size.set("width", x.to_value(lua))?;
                size.set("height", y.to_value(lua))?;
            }
            NuiDimension::XandY(xy) => {
                size.set("width", xy.to_value(lua))?;
                size.set("height", xy.to_value(lua))?;
            }
        }

        let position = lua.create_table()?;

        match self.position {
            NuiDimension::XorY(x, y) => {
                position.set("row", x.to_value(lua))?;
                position.set("col", y.to_value(lua))?;
            }
            NuiDimension::XandY(xy) => {
                position.set("row", xy.to_value(lua))?;
                position.set("col", xy.to_value(lua))?;
            }
        }

        out.set("size", size)?;
        out.set("position", position)?;
        out.set("enter", self.enter)?;
        out.set("focusable", self.focusable)?;
        out.set("zindex", self.zindex)?;
        out.set("relative", lua.to_value_with(&self.relative, ser_opts)?)?;
        out.set("border", lua.to_value_with(&self.border, ser_opts)?)?;
        out.set(
            "win_options",
            lua.to_value_with(&self.win_options, ser_opts)?,
        )?;
        out.set(
            "buf_options",
            lua.to_value_with(&self.buf_options, ser_opts)?,
        )?;

        Ok(LuaValue::Table(out))
    }
}

#[derive(Debug, Clone)]
pub struct NuiPopup<'lua>(pub LuaTable<'lua>);

impl<'lua> Deref for NuiPopup<'lua> {
    type Target = LuaTable<'lua>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<'lua> DerefMut for NuiPopup<'lua> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

unsafe impl Sync for NuiPopup<'_> {}
unsafe impl Send for NuiPopup<'_> {}

#[derive(Clone, Copy)]
pub enum NuiPopupFn {
    Close,
}

impl<'a> NuiPopup<'a> {
    pub fn mount(&self, lua: &'a Lua) -> LuaResult<()> {
        let mount: LuaFunction = self.get("mount")?;

        mount.call::<_, ()>(self.0.clone())
    }

    pub fn unmount(&self, lua: &'a Lua) -> LuaResult<()> {
        let mount: LuaFunction = self.get("unmount")?;

        mount.call::<_, ()>(self.0.clone())
    }

    pub fn on(
        &'a self,
        lua: &'a Lua,
        events: &[AutoCmdEvent],
        callback: LuaFunction<'a>,
    ) -> LuaResult<()> {
        let on: LuaFunction = self.get("on")?;

        let mut cmd_events = lua.create_table()?;

        for au_cmd in events.iter() {
            cmd_events.push(au_cmd.to_string());
        }

        on.call::<_, ()>((self.0.clone(), cmd_events, callback))
    }

    pub fn bufnr(&self, lua: &Lua) -> LuaResult<Option<u32>> {
        self.get("bufnr")
    }

    pub fn map(
        &'a self,
        lua: &'a Lua,
        mode: Mode,
        lhs: &str,
        rhs: LuaFunction<'a>,
        silent: bool,
    ) -> LuaResult<()> {
        let bufnr = self.bufnr(lua)?;

        let opts = KeymapOpts {
            silent: Some(silent),
            buffer: bufnr
        };

        NeoApi::set_keymap(lua, mode, lhs, rhs, opts)
    }
}

const NUI_POPUPS: &str = "nui_popups";

/// If you wish to use NuiApi please call init somewhere before using it (once)
pub struct NuiApi;

impl<'lua> NuiApi {
    pub fn init(lua: &'lua Lua) -> LuaResult<()> {
        let globals = lua.globals();
        globals.set(NUI_POPUPS, lua.create_table()?)
    }

    pub fn add_popup(lua: &'lua Lua, id: &str, popup: &LuaTable) -> LuaResult<()> {
        let popups: LuaTable = lua.globals().get(NUI_POPUPS)?;

        popups.set(id, popup)
    }

    pub fn get_popup(lua: &'lua Lua, id: &str) -> LuaResult<NuiPopup<'lua>> {
        let popups: LuaTable = lua.globals().get(NUI_POPUPS)?;

        Ok(NuiPopup(popups.get(id)?))
    }

    pub fn create_popup(
        lua: &'lua Lua,
        popup_opts: NuiPopupOpts,
        popup_id: &str,
    ) -> LuaResult<NuiPopup<'lua>> {
        let nui_popup: LuaTable = lua.load(r#"return require("nui.popup")"#).eval()?;

        let new: LuaFunction = nui_popup.get("new")?;

        let popup = new.call::<_, LuaTable>((nui_popup, popup_opts))?;

        Self::add_popup(lua, popup_id, &popup)?;

        Ok(NuiPopup(popup))
    }
}
