use crate::neo_api_types::{
    AutoCmd, AutoCmdEvent, AutoCmdOpts, ExtmarkOpts, LogLevel, Mode, OpenIn, OptValueType,
    StdpathType, Ui,
};
use crate::window::NeoWindow;
use crate::{CmdOpts, KeymapOpts};

use mlua::Lua;
use mlua::{
    prelude::{LuaFunction, LuaResult, LuaTable, LuaValue},
    IntoLua,
};
use std::fmt;
use std::path::{Path, PathBuf};

pub struct NeoApi;

#[allow(unused)]
impl NeoApi {
    pub fn delay<'a>(lua: &'a Lua, ms: u32, callback: LuaFunction<'a>) -> LuaResult<()> {
        let lfn: LuaFunction = lua.load("vim.defer_fn").eval()?;

        lfn.call((callback, ms))
    }

    /**
    Displays a notification to the user.

    This function can be overridden by plugins to display notifications using
    a custom provider (such as the system notification provider). By default,
    writes to |:messages|.

    Parameters: ~
      • {msg}    Content of the notification to show to the user.
    */
    pub fn notify(lua: &Lua, display: &impl fmt::Display) -> LuaResult<()> {
        let lfn: LuaFunction = lua.load("vim.notify").eval()?;

        lfn.call(display.to_string())
    }

    pub fn get_current_line(lua: &Lua) -> LuaResult<String> {
        let lfn: LuaFunction = lua.load("vim.api.nvim_get_current_line").eval()?;

        lfn.call(())
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
    pub fn notify_level(lua: &Lua, display: &impl fmt::Display, level: LogLevel) -> LuaResult<()> {
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

    pub fn cmd(lua: &Lua, opts: CmdOpts<'_>) -> LuaResult<()> {
        let cmd = format!("vim.cmd.{}", opts.cmd);
        let lfn: LuaFunction = lua.load(cmd).eval()?;

        lfn.call((opts))
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

        lfn.call((mode.get_str(), lhs, rhs, keymap_opts))
    }

    pub fn create_augroup(lua: &Lua, name: &str, clear: bool) -> LuaResult<u32> {
        let lfn: LuaFunction = lua.load("vim.api.nvim_create_augroup").eval()?;

        let opts = lua.create_table()?;
        opts.set("clear", clear)?;

        lfn.call((name, opts))
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
