use crate::buffer::NeoBuffer;
use crate::neo_api_types::{
    AutoCmd, AutoCmdEvent, AutoCmdOpts, ExtmarkOpts, LogLevel, Mode, OpenIn, OptValueType,
    StdpathType, Ui,
};
use crate::window::NeoWindow;
use crate::KeymapOpts;

use mlua::Lua;
use mlua::{
    prelude::{LuaError, LuaFunction, LuaResult, LuaTable, LuaValue},
    IntoLua,
};
use std::fmt;
use std::path::{Path, PathBuf};

pub struct NeoApi;

#[allow(unused)]
impl NeoApi {
    pub fn delay<'a>(lua: &'a Lua, ms: u32, callback: LuaFunction<'a>) -> LuaResult<()> {
        let fn_str = r#"
            function(timeout, callback)
                local timer = vim.uv.new_timer()
                timer:start(timeout, 0, vim.schedule_wrap(callback))
            end
        "#;

        let lfn: LuaFunction = lua.load(fn_str).eval()?;

        lfn.call((ms, callback))
    }

    /**
    Displays a notification to the user.

    This function can be overridden by plugins to display notifications using
    a custom provider (such as the system notification provider). By default,
    writes to |:messages|.

    Parameters: ~
      • {msg}    Content of the notification to show to the user.
    */
    pub fn notify(lua: &mlua::Lua, display: &impl fmt::Display) -> LuaResult<()> {
        let lfn: LuaFunction = lua.load("vim.notify").eval()?;

        let result = display.to_string();

        lfn.call::<_, ()>(result)
    }

    /**
    Displays a notification to the user.

    This function can be overridden by plugins to display notifications using
    a custom provider (such as the system notification provider). By default,
    writes to |:messages|.

    Parameters: ~
      • {msg}    Content of the notification to show to the user.
    */
    pub fn notify_dbg(lua: &mlua::Lua, debug: &impl fmt::Debug) -> LuaResult<()> {
        let lfn: LuaFunction = lua.load("vim.notify").eval()?;

        lfn.call(format!("{debug:?}"))
    }

    /**
    Displays a notification to the user.

    This function can be overridden by plugins to display notifications using
    a custom provider (such as the system notification provider). By default,
    writes to |:messages|.

    Parameters: ~
      • {msg}    Content of the notification to show to the user.
      • {level}  A log level
    */
    pub fn notify_level(
        lua: &Lua,
        display: &impl fmt::Display,
        level: LogLevel,
    ) -> LuaResult<()> {
        let lfn: LuaFunction = lua.load("vim.notify").eval()?;

        lfn.call((display.to_string(), level as usize))
    }

    /// Gets a human-readable representation of the given object.
    pub fn inspect<'lua, V: IntoLua<'lua>>(lua: &'lua Lua, table: V) -> LuaResult<()> {
        let ltb: LuaTable = lua.load("vim.inspect").eval()?;

        let lfn: LuaFunction = ltb.get("inspect")?;
        let result = lfn.call::<_, String>(table)?;

        Self::notify(lua, &result)
    }

    /**
    Sets the value of an option. The behavior of this function matches that of
    |:set|: for global-local options, both the global and local value are set
    unless otherwise specified with {scope}.

    Note the options {win} and {buf} cannot be used together.

    Parameters: ~
      • {name}   Option name
      • {value}  New option value
      • {opts}   Optional parameters
                 • scope: One of "global" or "local". Analogous to
                   |:setglobal| and |:setlocal|, respectively.
                 • win: |window-ID|. Used for setting window local option.
                 • buf: Buffer number. Used for setting buffer local option.
    */
    pub fn set_option_value<'a, V: IntoLua<'a>>(
        lua: &'a mlua::Lua,
        key: &str,
        value: V,
        opt_type: OptValueType,
    ) -> LuaResult<()> {
        let lfn: LuaFunction = lua.load("vim.api.nvim_set_option_value").eval()?;

        let opts = lua.create_table()?;

        match opt_type {
            OptValueType::Window(window) => opts.set("win", window.id())?,
            OptValueType::Buffer(buffer) => opts.set("buf", buffer.id())?,
        }

        lfn.call((key, value, opts))
    }

    pub fn get_current_win(lua: &mlua::Lua) -> LuaResult<NeoWindow> {
        let lfn: LuaFunction = lua.load("vim.api.nvim_get_current_win").eval()?;
        let win_id = lfn.call(())?;

        Ok(NeoWindow::new(win_id))
    }

    pub fn set_current_buf(lua: &mlua::Lua, buf_id: u32) -> LuaResult<()> {
        let lfn: mlua::Function = lua.load("vim.api.nvim_set_current_buf").eval()?;

        lfn.call(buf_id)
    }

    /**
    Sets the current window

    Attributes: ~
    &emsp; not allowed when |textlock| is active or in the |cmdwin|
    */
    pub fn set_current_win(lua: &Lua, win_id: u32) -> LuaResult<()> {
        let lfn: LuaFunction = lua.load("vim.api.nvim_set_current_win").eval()?;

        lfn.call(win_id)
    }

    /// This will create an empty buffer. Most of the time use the create_buf command
    pub fn open_new_buf(lua: &Lua, bang: bool) -> LuaResult<()> {
        let table = lua.create_table()?;
        table.set("bang", bang);

        let lfn: LuaFunction = lua.load("vim.cmd.enew").eval()?;

        lfn.call(table)
    }

    pub fn open_file(lua: &Lua, open_in: OpenIn, path: &str) -> LuaResult<()> {
        let lfn: LuaFunction = lua.load(format!("vim.cmd.{open_in}")).eval()?;

        lfn.call(path)
    }

    pub fn set_cwd(lua: &Lua, path: &Path) -> LuaResult<()> {
        let lfn: LuaFunction = lua.load("vim.api.nvim_set_current_dir").eval()?;

        lfn.call(path.to_string_lossy().to_string())
    }

    pub fn get_cwd(lua: &Lua) -> LuaResult<PathBuf> {
        let lfn: LuaFunction = lua.load("vim.fn.getcwd").eval()?;

        Ok(lfn.call::<_, String>(())?.into())
    }

    pub fn get_filepath(lua: &Lua) -> LuaResult<PathBuf> {
        let lfn: LuaFunction = lua.load("vim.fn.expand").eval()?;

        Ok(lfn.call::<_, String>("%:p")?.into())
    }

    pub fn get_filename(lua: &Lua) -> LuaResult<String> {
        let lfn: LuaFunction = lua.load("vim.fn.expand").eval()?;

        lfn.call::<_, String>("%:p:t")
    }

    pub fn get_filedir(lua: &Lua) -> LuaResult<PathBuf> {
        let lfn: LuaFunction = lua.load("vim.fn.expand").eval()?;

        Ok(lfn.call::<_, String>("%:p:h")?.into())
    }

    pub fn set_cmd_file(lua: &Lua, name: impl Into<String>) -> LuaResult<()> {
        let lfn: mlua::Function = lua.load("vim.cmd.file").eval()?;

        lfn.call::<_, ()>(name.into())
    }

    /**
    Returns |standard-path| locations of various default files and directories.

    What          Type     Description
    cache         String   Cache directory: arbitrary temporary storage for plugins, etc.
    config        String   User configuration directory. |init.vim| is stored here.
    config_dirs   List     Other configuration directories. (TODO)
    data          String   User data directory.
    data_dirs     List     Other data directories. (TODO)
    log           String   Logs directory (for use by plugins too).
    run           String   Run directory: temporary, local storage for sockets, named pipes, etc.
    state         String   Session state directory: storage for file drafts, swap, undo, |shada|.
    */
    pub fn stdpath(lua: &Lua, stdpath: StdpathType) -> LuaResult<PathBuf> {
        let lfn: LuaFunction = lua.load("vim.fn.stdpath").eval()?;

        Ok(lfn.call::<_, String>(stdpath.to_string())?.into())
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

    /**
    Adds a highlight to buffer.

    Useful for plugins that dynamically generate highlights to a buffer (like
    a semantic highlighter or linter). The function adds a single highlight to
    a buffer. Unlike |matchaddpos()| highlights follow changes to line
    numbering (as lines are inserted/removed above the highlighted line), like
    signs and marks do.

    Namespaces are used for batch deletion/updating of a set of highlights. To
    create a namespace, use |nvim_create_namespace()| which returns a
    namespace id. Pass it in to this function as `ns_id` to add highlights to
    the namespace. All highlights in the same namespace can then be cleared
    with single call to |nvim_buf_clear_namespace()|. If the highlight never
    will be deleted by an API call, pass `ns_id = -1`.

    As a shorthand, `ns_id = 0` can be used to create a new namespace for the
    highlight, the allocated id is then returned. If `hl_group` is the empty
    string no highlight is added, but a new `ns_id` is still returned. This is
    supported for backwards compatibility, new code should use
    |nvim_create_namespace()| to create a new empty namespace.

    Parameters: ~
      • {buffer}     Buffer handle, or 0 for current buffer
      • {ns_id}      namespace to use or -1 for ungrouped highlight
      • {hl_group}   Name of the highlight group to use
      • {line}       Line to highlight (zero-indexed)
      • {col_start}  Start of (byte-indexed) column range to highlight
      • {col_end}    End of (byte-indexed) column range to highlight, or -1 to
                     highlight to end of line

    Return: ~
        The ns_id that was used
    */
    pub fn buf_add_highlight(
        lua: &Lua,
        buf_id: u32,
        ns_id: i32,
        hl_group: &str,
        line: usize,
        col_start: u32,
        col_end: i32,
    ) -> LuaResult<i32> {
        let lfn: LuaFunction = lua.load("vim.api.nvim_buf_add_highlight").eval()?;

        lfn.call((buf_id, ns_id, hl_group, line, col_start, col_end))
    }

    /**
    Clears |namespace|d objects (highlights, |extmarks|, virtual text) from a
    region.

    Lines are 0-indexed. |api-indexing| To clear the namespace in the entire
    buffer, specify line_start=0 and line_end=-1.

    Parameters: ~
      • {buffer}      Buffer handle, or 0 for current buffer
      • {ns_id}       Namespace to clear, or -1 to clear all namespaces.
      • {line_start}  Start of range of lines to clear
      • {line_end}    End of range of lines to clear (exclusive) or -1 to
                      clear to end of buffer.
    */
    pub fn buf_clear_namespace(
        lua: &Lua,
        buf_id: u32,
        ns_id: i32,
        start: u32,
        end: i32,
    ) -> LuaResult<()> {
        let lfn: LuaFunction = lua.load("vim.api.nvim_buf_clear_namespace").eval()?;

        lfn.call((buf_id, ns_id, start, end))
    }

    /**
    Sets (replaces) a line-range in the buffer.

    Indexing is zero-based, end-exclusive. Negative indices are interpreted as
    length+1+index: -1 refers to the index past the end. So to change or
    delete the last element use start=-2 and end=-1.

    To insert lines at a given index, set `start` and `end` to the same index.
    To delete a range of lines, set `replacement` to an empty array.

    Out-of-bounds indices are clamped to the nearest valid value, unless
    `strict_indexing` is set.

    Attributes: ~
        not allowed when |textlock| is active

    Parameters: ~
      • {buffer}           Buffer handle, or 0 for current buffer
      • {start}            First line index
      • {end}              Last line index, exclusive
      • {strict_indexing}  Whether out-of-bounds should be an error.
      • {replacement}      Array of lines to use as replacement

    See also: ~
      • |nvim_buf_set_text()|
    */
    pub fn buf_set_lines(
        lua: &Lua,
        buf_id: u32,
        start: i32,
        end: i32,
        strict_indexing: bool,
        lines: &[String],
    ) -> LuaResult<()> {
        let lfn: LuaFunction = lua.load("vim.api.nvim_buf_set_lines").eval()?;

        lfn.call::<_, ()>((buf_id, start, end, strict_indexing, lines))
    }

    pub fn buf_get_lines(
        lua: &Lua,
        buf_id: u32,
        start: i32,
        end: i32,
        strict_indexing: bool,
    ) -> LuaResult<Vec<String>> {
        let lfn: LuaFunction = lua.load("vim.api.nvim_buf_get_lines").eval()?;

        lfn.call((buf_id, start, end, strict_indexing))
    }

    pub fn list_uis(lua: &mlua::Lua) -> LuaResult<Vec<Ui>> {
        let lfn: LuaFunction = lua.load("vim.api.nvim_list_uis").eval()?;

        lfn.call(())
    }

    /**
    Creates or updates an |extmark|.

    By default a new extmark is created when no id is passed in, but it is
    also possible to create a new mark by passing in a previously unused id or
    move an existing mark by passing in its id. The caller must then keep
    track of existing and unused ids itself. (Useful over RPC, to avoid
    waiting for the return value.)

    Using the optional arguments, it is possible to use this to highlight a
    range of text, and also to associate virtual text to the mark.

    If present, the position defined by `end_col` and `end_row` should be
    after the start position in order for the extmark to cover a range. An
    earlier end position is not an error, but then it behaves like an empty
    range (no highlighting).

    Parameters: ~
      • {buffer}  Buffer handle, or 0 for current buffer
      • {ns_id}   Namespace id from |nvim_create_namespace()|
      • {line}    Line where to place the mark, 0-based. |api-indexing|
      • {col}     Column where to place the mark, 0-based. |api-indexing|
      • {opts}    Optional parameters.
    */
    pub fn buf_set_extmark(
        lua: &Lua,
        buf_id: u32,
        ns_id: u32,
        line: u32,
        col: u32,
        opts: ExtmarkOpts,
    ) -> mlua::Result<()> {
        let lfn: LuaFunction = lua.load("vim.api.nvim_buf_set_extmark").eval()?;
        let opts: LuaValue = opts.into_lua(lua)?;

        lfn.call::<_, ()>((buf_id, ns_id, line, col, opts))
    }

    pub fn set_keymap<'a>(
        lua: &'a mlua::Lua,
        mode: Mode,
        lhs: &'a str,
        rhs: mlua::Function,
        keymap_opts: KeymapOpts,
    ) -> mlua::Result<()> {
        let lfn: LuaFunction = lua.load("vim.keymap.set").eval()?;

        lfn.call::<_, ()>((mode.get_str(), lhs, rhs, keymap_opts))
    }

    /// Creates an |autocommand| event handler, defined by `callback`
    pub fn create_autocmd<'a>(
        lua: &'a Lua,
        events: &[AutoCmdEvent],
        opts: AutoCmdOpts<'a>,
    ) -> LuaResult<AutoCmd> {
        let lfn: LuaFunction = lua.load("vim.api.nvim_create_autocmd").eval()?;

        let events: Vec<String> = events.iter().map(|e| e.to_string()).collect();

        let id = lfn.call::<_, u32>((events, opts))?;

        Ok(AutoCmd::new(id))
    }

    pub fn set_insert_mode(lua: &Lua, insert: bool) -> LuaResult<()> {
        if insert {
            let lfn: LuaFunction = lua.load("vim.cmd.startinsert").eval()?;

            lfn.call(())
        } else {
            let lfn: LuaFunction = lua.load("vim.cmd.stopinsert").eval()?;

            lfn.call(())
        }
    }
}
