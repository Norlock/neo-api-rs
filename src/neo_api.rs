use crate::neo_api_types::{
    AutoCmd, AutoCmdEvent, AutoCmdOpts, ExtmarkOpts, LogLevel, Mode, OpenIn, OptValueType,
    StdpathType, Ui,
};
use crate::{CmdOpts, FileTypeMatch, KeymapOpts, NeoDebug};
use crate::{NeoWindow, RTM};
use mlua::{
    prelude::{LuaFunction, LuaResult, LuaTable, LuaValue},
    FromLua, IntoLua, Lua,
};
use std::fmt;
use std::path::{Path, PathBuf};

pub struct NeoApi;

#[allow(unused)]
impl NeoApi {
    pub fn init(lua: &Lua) -> LuaResult<()> {
        //DevIcon::init(lua)?;

        let cb = lua.create_async_function(|lua, ()| async {
            RTM.block_on(NeoDebug::display(lua));

            Ok(())
        })?;

        Self::create_user_command(lua, "NeoApiShowLogs", cb, false)?;

        let cb = lua.create_async_function(|lua, ()| async {
            RTM.block_on(NeoDebug::clear_logs());

            Ok(())
        })?;

        Self::create_user_command(lua, "NeoApiClearLogs", cb, false)
    }

    pub fn create_user_command<'a>(
        lua: &'a Lua,
        name: &str,
        callback: LuaFunction<'a>,
        bang: bool,
    ) -> LuaResult<()> {
        let lfn: LuaFunction = lua.load("vim.api.nvim_create_user_command").eval()?;

        let opts = lua.create_table()?;
        opts.set("bang", bang)?;

        lfn.call((name, callback, opts))
    }

    pub fn delay<'a>(lua: &'a Lua, ms: u32, callback: LuaFunction<'a>) -> LuaResult<()> {
        let lfn: LuaFunction = lua.load("vim.defer_fn").eval()?;

        lfn.call((callback, ms))
    }

    pub fn schedule_wrap<'a>(
        lua: &'a Lua,
        callback: LuaFunction<'a>,
    ) -> LuaResult<LuaFunction<'a>> {
        let lfn: LuaFunction = lua.load("vim.schedule_wrap").eval()?;

        lfn.call(callback)
    }

    /// timer_id will be prefixed with neo_timer_ and stored in globals.
    /// In the callback use try lock to prevent async errors on mlua
    pub fn start_interval(
        lua: &Lua,
        timer_id: &str,
        ms: u32,
        callback: LuaFunction<'_>,
    ) -> LuaResult<()> {
        let lfn: LuaFunction = lua.load("vim.uv.new_timer").eval()?;

        let timer: LuaValue = lfn.call(())?;

        let lfn: LuaFunction = lua.load("vim.uv.timer_start").eval()?;

        let callback = Self::schedule_wrap(lua, callback)?;

        lua.globals()
            .set(format!("neo_timer_{timer_id}"), timer.clone())?;

        lfn.call((timer, ms, ms, callback))
    }

    pub fn stop_interval(lua: &Lua, timer_id: &str) -> LuaResult<()> {
        let timer: LuaValue = lua.globals().get(format!("neo_timer_{timer_id}"))?;

        let lfn: LuaFunction = lua.load("vim.uv.timer_stop").eval()?;

        lfn.call(timer)
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

    pub fn get_option_value<'a, V: FromLua<'a>>(
        lua: &'a mlua::Lua,
        key: &str,
        opt_type: OptValueType,
    ) -> LuaResult<V> {
        let lfn: LuaFunction = lua.load("vim.api.nvim_get_option_value").eval()?;

        let opts = lua.create_table()?;

        match opt_type {
            OptValueType::Window(window) => opts.set("win", window.id())?,
            OptValueType::Buffer(buffer) => opts.set("buf", buffer.id())?,
        }

        lfn.call((key, opts))
    }

    /**
    Perform filetype detection.

    The filetype can be detected using one of three methods:
    1. Using an existing buffer
    2. Using only a file name
    3. Using only file contents

    Of these, option 1 provides the most accurate result as it uses both the
    buffer's filename and (optionally) the buffer contents. Options 2 and 3
    can be used without an existing buffer, but may not always provide a match
    in cases where the filename (or contents) cannot unambiguously determine
    the filetype.

    Each of the three options is specified using a key to the single argument
    of this function. Example: >lua
    */
    pub fn filetype_match(lua: &Lua, opts: FileTypeMatch) -> LuaResult<Option<String>> {
        let lfn: LuaFunction = lua.load("vim.filetype.match").eval()?;

        lfn.call(opts)
    }

    pub fn get_current_win(lua: &Lua) -> LuaResult<NeoWindow> {
        let lfn: LuaFunction = lua.load("vim.api.nvim_get_current_win").eval()?;
        let win_id = lfn.call(())?;

        Ok(NeoWindow::new(win_id))
    }

    pub fn set_current_buf(lua: &Lua, buf_id: u32) -> LuaResult<()> {
        let lfn: LuaFunction = lua.load("vim.api.nvim_set_current_buf").eval()?;

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

    pub fn stop_lsp(lua: &Lua) -> LuaResult<()> {
        let lfn_gc: LuaFunction = lua.load("vim.lsp.get_clients").eval()?;
        let lfn_sc: LuaFunction = lua.load("vim.lsp.stop_client").eval()?;

        let clients: LuaValue = lfn_gc.call(())?;

        lfn_sc.call(clients)
    }

    pub fn cmd(lua: &Lua, opts: CmdOpts<'_>) -> LuaResult<()> {
        let cmd = format!("vim.cmd.{}", opts.cmd);
        let lfn: LuaFunction = lua.load(cmd).eval()?;

        lfn.call(opts)
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

    pub fn del_augroup_by_name(lua: &Lua, name: &str) -> LuaResult<()> {
        let lfn: LuaFunction = lua.load("vim.api.nvim_del_augroup_by_name").eval()?;

        lfn.call(name)
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

    pub fn get_dev_icon(
        lua: &Lua,
        filename: &str,
        extension: &str,
    ) -> LuaResult<(Option<String>, Option<String>)> {
        let lfn: LuaFunction = lua.load("require('nvim-web-devicons').get_icon").eval()?;

        lfn.call((filename, extension))
    }
}
