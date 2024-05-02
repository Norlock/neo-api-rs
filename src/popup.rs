use crate::{NeoApi, NeoWindow, TextType};

use mlua::{
    prelude::{LuaFunction, LuaResult, LuaValue},
    IntoLua, Lua,
};
use serde::Serialize;
use std::fmt;

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub enum PopupRelative {
    #[default]
    Win,
    Cursor,
    Editor,
    Mouse,
}

impl fmt::Display for PopupRelative {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Win => f.write_str("win"),
            Self::Cursor => f.write_str("cursor"),
            Self::Mouse => f.write_str("mouse"),
            Self::Editor => f.write_str("editor"),
        }
    }
}

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub enum Anchor {
    #[default]
    NorthWest,
    NorthEast,
    SouthWest,
    SouthEast,
}

impl fmt::Display for Anchor {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::NorthWest => f.write_str("NW"),
            Self::NorthEast => f.write_str("NE"),
            Self::SouthWest => f.write_str("SW"),
            Self::SouthEast => f.write_str("SE"),
        }
    }
}

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub enum PopupStyle {
    #[default]
    Minimal,
}

impl fmt::Display for PopupStyle {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("minimal")
    }
}

#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub enum PopupBorder {
    #[default]
    None,
    Single,
    Double,
    Rounded,
    Solid,
    Shadow,
    Custom(Vec<String>),
}

impl fmt::Display for PopupBorder {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::None => f.write_str("none"),
            Self::Single => f.write_str("single"),
            Self::Double => f.write_str("double"),
            Self::Rounded => f.write_str("rounded"),
            Self::Solid => f.write_str("solid"),
            Self::Shadow => f.write_str("shadow"),
            Self::Custom(cus) => f.write_str(&cus.join(",")),
        }
    }
}

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub enum PopupAlign {
    #[default]
    Left,
    Center,
    Right,
}

impl fmt::Display for PopupAlign {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Left => f.write_str("left"),
            Self::Center => f.write_str("center"),
            Self::Right => f.write_str("right"),
        }
    }
}

#[derive(Debug, Default, Serialize, Clone, Copy, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum PopupSplit {
    #[default]
    Left,
    Right,
    Above,
    Below,
}

impl fmt::Display for PopupSplit {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Left => f.write_str("left"),
            Self::Right => f.write_str("right"),
            Self::Above => f.write_str("above"),
            Self::Below => f.write_str("below"),
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub enum PopupSize {
    Fixed(i32),
    /// Between 0 and 1
    Percentage(f32),
}

#[derive(Default)]
pub struct WinOptions {
    //  width: Window width (in character cells). Minimum of 1.
    pub width: Option<PopupSize>,
    pub height: Option<PopupSize>,
    pub col: Option<PopupSize>,
    pub row: Option<PopupSize>,
    pub relative: PopupRelative,
    pub buf_pos: Option<(u32, u32)>,
    pub win: Option<u32>,
    pub anchor: Anchor,
    pub focusable: bool,
    pub external: bool,
    pub zindex: u32,
    pub style: Option<PopupStyle>,
    pub border: PopupBorder,
    pub title: Option<TextType>,
    pub title_pos: PopupAlign,
    pub footer: Option<TextType>,
    pub footer_pos: PopupAlign,
    pub noautocmd: bool,
    pub fixed: bool,
    pub hide: bool,
    pub vertical: bool,
    pub split: PopupSplit,
}

impl<'a> IntoLua<'a> for WinOptions {
    fn into_lua(self, lua: &'a Lua) -> LuaResult<LuaValue<'a>> {
        let out = lua.create_table()?;

        let uis = NeoApi::list_uis(lua)?;
        let ui = &uis[0];

        let mut raw_width = 40;
        let mut raw_height = 40;
        let mut raw_col = 0;
        let mut raw_row = 0;

        if let Some(width) = self.width {
            match width {
                PopupSize::Fixed(width) => {
                    raw_width = width;
                }
                PopupSize::Percentage(percentage) => {
                    raw_width = (ui.width as f32 * percentage).round() as i32;
                }
            }
        }

        if let Some(height) = self.height {
            match height {
                PopupSize::Fixed(height) => {
                    raw_height = height;
                }
                PopupSize::Percentage(percentage) => {
                    raw_height = (ui.height as f32 * percentage).round() as i32;
                }
            }
        }

        if let Some(row) = self.row {
            match row {
                PopupSize::Fixed(row) => {
                    raw_row = row;
                }
                PopupSize::Percentage(percentage) => {
                    raw_row = ((ui.height as i32 - raw_height) as f32 * percentage) as i32;
                }
            }
        }

        if let Some(col) = self.col {
            match col {
                PopupSize::Fixed(col) => {
                    raw_col = col;
                }
                PopupSize::Percentage(percentage) => {
                    raw_col = ((ui.width as i32 - raw_width) as f32 * percentage) as i32;
                }
            }
        }

        out.set("relative", self.relative.to_string())?;
        out.set("width", raw_width)?;
        out.set("height", raw_height)?;
        out.set("row", raw_row)?;
        out.set("col", raw_col)?;
        out.set("anchor", self.anchor.to_string())?;

        if let Some(style) = self.style {
            out.set("style", style.to_string())?;
        }

        out.set("border", self.border.to_string())?;

        if let Some(title) = self.title {
            out.set("title", title.into_lua(lua)?)?;
            out.set("title_pos", self.title_pos.to_string())?;
        }

        if let Some(footer) = self.footer {
            out.set("footer", footer.into_lua(lua)?)?;
            out.set("footer_pos", self.footer_pos.to_string())?;
        }

        out.set("noautocmd", self.noautocmd)?;

        Ok(LuaValue::Table(out))
    }
}

pub struct NeoPopup;

impl NeoPopup {
    pub fn open_win(
        lua: &Lua,
        buf_id: u32,
        enter: bool,
        config: WinOptions,
    ) -> LuaResult<NeoWindow> {
        let lfn: LuaFunction = lua.load("vim.api.nvim_open_win").eval()?;

        let win_id = lfn.call::<_, u32>((buf_id, enter, config.into_lua(lua)?))?;

        Ok(NeoWindow::new(win_id))
    }
}
