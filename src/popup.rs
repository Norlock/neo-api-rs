use crate::{HLText, NeoApi, NeoBuffer, NeoWindow, TextType};
use mlua::{
    prelude::{LuaFunction, LuaResult, LuaValue},
    FromLua, IntoLua, Lua,
};
use std::{
    fmt::{self, Display},
    time::Duration,
};

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

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
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
    Fixed(u32),
    /// Between 0 and 1
    Percentage(f32),
}

#[derive(Default)]
pub struct WinOptions {
    /// width: Window width (in character cells). Minimum of 1.
    pub width: Option<PopupSize>,
    /// height: Window height (in character cells). Minimum of 1.
    pub height: Option<PopupSize>,
    /// col: Column position in units of "screen cell width", may be fractional.
    pub col: Option<PopupSize>,
    /// row: Row position in units of "screen cell height", may be fractional.
    pub row: Option<PopupSize>,
    /**
    relative: Sets the window layout to "floating", placed at (row,col) coordinates relative to:
    • "editor" The global editor grid
    • "win" Window given by the `win` field, or currentwindow.
    • "cursor" Cursor position in current window.
    • "mouse" Mouse position
    */
    pub relative: PopupRelative,
    pub buf_pos: Option<(u32, u32)>,
    ///  win: |window-ID| window to split, or relative window when creating a float (relative="win").
    pub win: Option<u32>,
    pub anchor: Anchor,
    /**
      focusable: Enable focus by user actions (wincmds, mouse
      events). Defaults to true. Non-focusable windows can be
      entered by |nvim_set_current_win()|.
    */
    pub focusable: Option<bool>,
    pub external: bool,
    /**
      zindex: Stacking order. floats with higher `zindex` go on
      top on floats with lower indices. Must be larger than
      zero. The following screen elements have hard-coded
      z-indices:
      • 100: insert completion popupmenu
      • 200: message scrollback
      • 250: cmdline completion popupmenu (when
      wildoptions+=pum) The default value for floats are 50.
      In general, values below 100 are recommended, unless
      there is a good reason to overshadow builtin elements.
    */
    pub zindex: u32,
    /**
      style: (optional) Configure the appearance of the window.
      Currently only supports one value:
      • "minimal" Nvim will display the window with many UI
      options disabled. This is useful when displaying a
      temporary float where the text should not be edited.
      Disables 'number', 'relativenumber', 'cursorline',
      'cursorcolumn', 'foldcolumn', 'spell' and 'list'
      options. 'signcolumn' is changed to `auto` and
      'colorcolumn' is cleared. 'statuscolumn' is changed to
      empty. The end-of-buffer region is hidden by setting
      `eob` flag of 'fillchars' to a space char, and clearing
      the |hl-EndOfBuffer| region in 'winhighlight'.
    */
    pub style: Option<PopupStyle>,
    /**
      border: Style of (optional) window border. This can either
      be a string or an array. The string values are
      • "none": No border (default).
      • "single": A single line box.
      • "double": A double line box.
      • "rounded": Like "single", but with rounded corners
      ("╭" etc.).
      • "solid": Adds padding by a single whitespace cell.
      • "shadow": A drop shadow effect by blending with the
      background.
      • If it is an array, it should have a length of eight or
      any divisor of eight. The array will specify the eight
      chars building up the border in a clockwise fashion
      starting with the top-left corner. As an example, the
      double box style could be specified as: >
      [ "╔", "═" ,"╗", "║", "╝", "═", "╚", "║" ].
      <
      If the number of chars are less than eight, they will be
      repeated. Thus an ASCII border could be specified as >
      [ "/", "-", \"\\\\\", "|" ],

      or all chars the same as >
      [ "x" ].

      An empty string can be used to turn off a specific border,
      for instance, >
      [ "", "", "", ">", "", "", "", "<" ]

      will only make vertical borders but not horizontal ones.
      By default, `FloatBorder` highlight is used, which links
      to `WinSeparator` when not defined. It could also be
      specified by character: >
      [ ["+", "MyCorner"], ["x", "MyBorder"] ].
    */
    pub border: PopupBorder,
    /**
      title: Title (optional) in window border, string or list.
      List should consist of `[text, highlight]` tuples. If
      string, the default highlight group is `FloatTitle`.
    */
    pub title: Option<TextType>,
    /**
      title_pos: Title position. Must be set with `title`
      option. Value can be one of "left", "center", or "right".
      Default is `"left"`.
    */
    pub title_pos: PopupAlign,
    /**
      footer: Footer (optional) in window border, string or
      list. List should consist of `[text, highlight]` tuples.
      If string, the default highlight group is `FloatFooter`.
    */
    pub footer: Option<TextType>,
    pub footer_pos: PopupAlign,

    /// noautocmd: If true then all autocommands are blocked for
    /// the duration of the call.
    pub noautocmd: bool,

    /// fixed: If true when anchor is NW or SW, the float window
    /// would be kept fixed even if the window would be truncated.
    pub fixed: bool,
    pub hide: bool,
    /// vertical: Split vertically |:vertical|.
    pub vertical: bool,
    /// split: Split direction: "left", "right", "above", "below".
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
                    raw_width = (ui.width as f32 * percentage) as u32;
                }
            }
        }

        if let Some(height) = self.height {
            match height {
                PopupSize::Fixed(height) => {
                    raw_height = height;
                }
                PopupSize::Percentage(percentage) => {
                    raw_height = (ui.height as f32 * percentage) as u32;
                }
            }
        }

        if let Some(row) = self.row {
            match row {
                PopupSize::Fixed(row) => {
                    raw_row = row;
                }
                PopupSize::Percentage(percentage) => {
                    raw_row = (ui.height as f32 * percentage) as u32;
                }
            }
        }

        if let Some(col) = self.col {
            match col {
                PopupSize::Fixed(col) => {
                    raw_col = col;
                }
                PopupSize::Percentage(percentage) => {
                    raw_col = (ui.width as f32 * percentage) as u32;
                }
            }
        }

        out.set("relative", self.relative.to_string())?;
        out.set("width", raw_width)?;
        out.set("height", raw_height)?;
        out.set("row", raw_row)?;
        out.set("col", raw_col)?;
        out.set("anchor", self.anchor.to_string())?;
        out.set("border", self.border.to_string())?;
        out.set("noautocmd", self.noautocmd)?;

        if let Some(style) = self.style {
            out.set("style", style.to_string())?;
        }

        if let Some(focusable) = self.focusable {
            out.set("focusable", focusable)?;
        }

        if let Some(title) = self.title {
            out.set("title", title.into_lua(lua)?)?;
            out.set("title_pos", self.title_pos.to_string())?;
        }

        if let Some(footer) = self.footer {
            out.set("footer", footer.into_lua(lua)?)?;
            out.set("footer_pos", self.footer_pos.to_string())?;
        }

        Ok(LuaValue::Table(out))
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct NeoPopup {
    pub win: NeoWindow,
    pub buf: NeoBuffer,
}

impl<'a> IntoLua<'a> for NeoPopup {
    fn into_lua(self, lua: &'a Lua) -> LuaResult<LuaValue<'a>> {
        let out = lua.create_table()?;
        out.set("win", self.win.id())?;
        out.set("buf", self.buf.id())?;

        Ok(LuaValue::Table(out))
    }
}

impl<'a> FromLua<'a> for NeoPopup {
    fn from_lua(value: LuaValue<'a>, _lua: &'a Lua) -> LuaResult<Self> {
        if let LuaValue::Table(table) = value {
            let win_id = table.get("win")?;
            let buf_id = table.get("buf")?;

            Ok(NeoPopup {
                win: NeoWindow::new(win_id),
                buf: NeoBuffer::new(buf_id),
            })
        } else {
            Err(mlua::Error::FromLuaConversionError {
                from: value.type_name(),
                to: "NeoPopup",
                message: None,
            })
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PopupLevel {
    Succes,
    Error,
    Neutral,
}

impl Display for PopupLevel {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Error => f.write_str("DiagnosticSignError"),
            Self::Succes => f.write_str("DiagnosticSignOk"),
            Self::Neutral => f.write_str("Normal"),
        }
    }
}

pub struct PopupNotify {
    pub level: PopupLevel,
    pub title: String,
    pub messages: Vec<String>,
    pub duration: Duration,
}

impl NeoPopup {
    /// This will create and open the popup, using `open_win` is also fine
    pub fn open(lua: &Lua, buf: NeoBuffer, enter: bool, config: WinOptions) -> LuaResult<Self> {
        let win = Self::open_win(lua, &buf, enter, config)?;
        Ok(Self { win, buf })
    }

    /// This will create the popup win.
    pub fn open_win(
        lua: &Lua,
        buf: &NeoBuffer,
        enter: bool,
        config: WinOptions,
    ) -> LuaResult<NeoWindow> {
        let lfn: LuaFunction = lua.load("vim.api.nvim_open_win").eval()?;

        let win_id = lfn.call((buf.id(), enter, config))?;

        Ok(NeoWindow::new(win_id))
    }

    pub fn notify(lua: &Lua, options: PopupNotify) -> LuaResult<()> {
        let popup_buf = NeoBuffer::create(lua, false, true)?;

        popup_buf.set_lines(lua, 0, -1, false, &options.messages)?;

        let popup_win = Self::open_win(
            lua,
            &popup_buf,
            false,
            WinOptions {
                relative: PopupRelative::Editor,
                width: Some(PopupSize::Fixed(50)),
                height: Some(PopupSize::Fixed(options.messages.len() as u32)),
                col: Some(PopupSize::Fixed(1000)),
                row: Some(PopupSize::Fixed(0)),
                style: Some(PopupStyle::Minimal),
                border: PopupBorder::Rounded,
                title: Some(TextType::Tuples(vec![HLText::new(
                    options.title,
                    options.level.to_string(),
                )])),
                title_pos: PopupAlign::Left,
                ..Default::default()
            },
        )?;

        let close_popup = lua.create_function(move |lua, _: ()| popup_win.close(lua, true))?;

        NeoApi::delay(lua, options.duration.as_millis() as u32, close_popup)
    }
}
