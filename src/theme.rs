use crate::mlua::prelude::{Lua, LuaFunction, LuaResult, LuaSerializeOptions, LuaValue};
use mlua::{IntoLua, LuaSerdeExt};

#[derive(Clone, Debug, Default, serde::Serialize)]
pub struct HLOpts {
    /// color name or "#RRGGBB"
    pub fg: Option<String>,
    /// color name or "#RRGGBB"
    pub bg: Option<String>,
    /// color name or "#RRGGBB"
    pub sp: Option<String>,
    /// Between 0 and 100
    pub blend: Option<u32>,

    /// Default false
    pub bold: bool,
    /// Default false
    pub standout: bool,
    /// Default false
    pub underline: bool,
    pub undercurl: bool,
    pub underdouble: bool,
    pub underdotted: bool,
    pub underdashed: bool,
    pub strikethrough: bool,
    pub italic: bool,
    pub reverse: bool,
    pub nocombine: bool,
    /// link: name of another highlight group to link to, see |:hi-link|.
    pub link: Option<String>,

    /// Don't override existing definition |:hi-default|
    pub default: bool,

    pub ctermfg: Option<String>,
    pub ctermbg: Option<String>,
    // pub cterm todo
    pub force: bool,
}

impl<'a> IntoLua<'a> for HLOpts {
    fn into_lua(self, lua: &'a Lua) -> LuaResult<LuaValue<'a>> {
        let mut ser_opts = LuaSerializeOptions::new();
        ser_opts.serialize_none_to_null = false;
        ser_opts.serialize_unit_to_null = false;

        lua.to_value_with(&self, ser_opts)
    }
}

pub struct NeoTheme;

impl NeoTheme {
    /**
    Sets a highlight group.

    Note: ~
      • Unlike the `:highlight` command which can update a highlight group,
        this function completely replaces the definition. For example:
        `nvim_set_hl(0, 'Visual', {})` will clear the highlight group
        'Visual'.
      • The fg and bg keys also accept the string values `"fg"` or `"bg"`
        which act as aliases to the corresponding foreground and background
        values of the Normal group. If the Normal group has not been defined,
        using these values results in an error.
      • If `link` is used in combination with other attributes; only the
        `link` will take effect (see |:hi-link|).

    Parameters: ~
      • {ns_id}  Namespace id for this highlight |nvim_create_namespace()|.
                 Use 0 to set a highlight group globally |:highlight|.
                 Highlights from non-global namespaces are not active by
                 default, use |nvim_set_hl_ns()| or |nvim_win_set_hl_ns()| to
                 activate them.
      • {name}   Highlight group name, e.g. "ErrorMsg"
      • {val}    Highlight definition map, accepts the following keys:
                 • fg: color name or "#RRGGBB", see note.
                 • bg: color name or "#RRGGBB", see note.
                 • sp: color name or "#RRGGBB"
                 • blend: integer between 0 and 100
                 • bold: boolean
                 • standout: boolean
                 • underline: boolean
                 • undercurl: boolean
                 • underdouble: boolean
                 • underdotted: boolean
                 • underdashed: boolean
                 • strikethrough: boolean
                 • italic: boolean
                 • reverse: boolean
                 • nocombine: boolean
                 • link: name of another highlight group to link to, see |:hi-link|.
                 • default: Don't override existing definition |:hi-default|
                 • ctermfg: Sets foreground of cterm color |ctermfg|
                 • ctermbg: Sets background of cterm color |ctermbg|
                 • cterm: cterm attribute map, like |highlight-args|. If not
                   set, cterm attributes will match those from the attribute
                   map documented above.
                 • force: if true force update the highlight group when it
                   exists.
    */
    pub fn set_hl(lua: &Lua, ns_id: u32, group_name: &str, opts: HLOpts) -> LuaResult<()> {
        let lfn: LuaFunction = lua.load("vim.api.nvim_set_hl").eval()?;

        lfn.call((ns_id, group_name, opts))
    }

    pub fn set_hl_ns(lua: &Lua, ns_id: u32) -> LuaResult<()> {
        let lfn: LuaFunction = lua.load("vim.api.nvim_set_hl_ns").eval()?;

        lfn.call(ns_id)
    }

    /**
    Creates a new namespace or gets an existing one.

    Namespaces are used for buffer highlights and virtual text, see
    |nvim_buf_add_highlight()| and |nvim_buf_set_extmark()|.

    Namespaces can be named or anonymous. If `name` matches an existing
    namespace, the associated id is returned. If `name` is an empty string a
    new, anonymous namespace is created.

    Parameters: ~
      • {name}  Namespace name or empty string

    Return: ~
        Namespace id
    */
    pub fn create_namespace(lua: &Lua, ns: &str) -> LuaResult<u32> {
        let lfn: LuaFunction = lua.load("vim.api.nvim_create_namespace").eval()?;

        lfn.call::<_, u32>(ns)
    }
}
