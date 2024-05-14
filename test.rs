#![feature(prelude_import)]
#[prelude_import]
use std::prelude::rust_2021::*;
#[macro_use]
extern crate std;
mod buffer {
    #![allow(unused)]
    use crate::neo_api::NeoApi;
    use crate::neo_api_types::{ExtmarkOpts, OptValueType};
    use crate::{BufferDeleteOpts, KeymapOpts, Mode};
    use mlua::prelude::{
        IntoLua, Lua, LuaError, LuaFunction, LuaResult, LuaTable, LuaValue,
    };
    pub struct NeoBuffer(u32);
    #[automatically_derived]
    impl ::core::clone::Clone for NeoBuffer {
        #[inline]
        fn clone(&self) -> NeoBuffer {
            let _: ::core::clone::AssertParamIsClone<u32>;
            *self
        }
    }
    #[automatically_derived]
    impl ::core::marker::Copy for NeoBuffer {}
    #[automatically_derived]
    impl ::core::fmt::Debug for NeoBuffer {
        #[inline]
        fn fmt(&self, f: &mut ::core::fmt::Formatter) -> ::core::fmt::Result {
            ::core::fmt::Formatter::debug_tuple_field1_finish(f, "NeoBuffer", &&self.0)
        }
    }
    #[automatically_derived]
    impl ::core::marker::StructuralPartialEq for NeoBuffer {}
    #[automatically_derived]
    impl ::core::cmp::PartialEq for NeoBuffer {
        #[inline]
        fn eq(&self, other: &NeoBuffer) -> bool {
            self.0 == other.0
        }
    }
    #[automatically_derived]
    impl ::core::cmp::Eq for NeoBuffer {
        #[inline]
        #[doc(hidden)]
        #[coverage(off)]
        fn assert_receiver_is_total_eq(&self) -> () {
            let _: ::core::cmp::AssertParamIsEq<u32>;
        }
    }
    #[automatically_derived]
    impl ::core::hash::Hash for NeoBuffer {
        #[inline]
        fn hash<__H: ::core::hash::Hasher>(&self, state: &mut __H) -> () {
            ::core::hash::Hash::hash(&self.0, state)
        }
    }
    impl NeoBuffer {
        pub const ZERO: Self = Self(0);
        /**
    Creates a new, empty, unnamed buffer.

    Parameters: ~
      • {listed}   Sets 'buflisted'
      • {scratch}  Creates a "throwaway" |scratch-buffer| for temporary work
                   (always 'nomodified'). Also sets 'nomodeline' on the
                   buffer.

    Return: ~
        Buffer handle, or 0 on error

    See also: ~
      • buf_open_scratch
    */
        pub fn create(lua: &Lua, listed: bool, scratch: bool) -> LuaResult<NeoBuffer> {
            let lfn: LuaFunction = lua.load("vim.api.nvim_create_buf").eval()?;
            let buf_id: u32 = lfn.call::<_, u32>((listed, scratch))?;
            if buf_id == 0 {
                return Err(LuaError::RuntimeError("Buffer not created".to_string()));
            }
            Ok(NeoBuffer::new(buf_id))
        }
        pub fn new(id: u32) -> Self {
            Self(id)
        }
        pub fn id(&self) -> u32 {
            self.0
        }
        pub fn get_current_buf(lua: &Lua) -> LuaResult<NeoBuffer> {
            let lfn: LuaFunction = lua.load("vim.api.nvim_get_current_buf").eval()?;
            let buf_id = lfn.call(())?;
            Ok(NeoBuffer::new(buf_id))
        }
        pub fn keymap_opts(&self, silent: bool) -> KeymapOpts {
            KeymapOpts {
                buffer: Some(self.0),
                silent: Some(silent),
            }
        }
        pub fn set_keymap<'a>(
            &self,
            lua: &'a Lua,
            mode: Mode,
            lhs: &str,
            rhs: LuaFunction<'a>,
        ) -> LuaResult<()> {
            NeoApi::set_keymap(lua, mode, lhs, rhs, self.keymap_opts(true))
        }
        /**
    Sets the current buffer.

    Attributes: ~
        not allowed when |textlock| is active or in the |cmdwin|
    */
        pub fn set_current(&self, lua: &mlua::Lua) -> LuaResult<()> {
            let lfn: mlua::Function = lua.load("vim.api.nvim_set_current_buf").eval()?;
            lfn.call(self.id())
        }
        /**
    Deletes the buffer. See |:bwipeout|

    Attributes: ~
        not allowed when |textlock| is active or in the |cmdwin|

    Parameters: ~
      • {buffer}  Buffer handle, or 0 for current buffer
      • {opts}    Optional parameters. Keys:
                  • force: Force deletion and ignore unsaved changes.
                  • unload: Unloaded only, do not delete. See |:bunload|
    */
        pub fn delete(&self, lua: &Lua, opts: BufferDeleteOpts) -> LuaResult<()> {
            let lfn: LuaFunction = lua.load("vim.api.nvim_buf_delete").eval()?;
            lfn.call::<_, ()>((self.id(), opts))
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
            NeoApi::set_option_value(lua, key, value, OptValueType::Buffer(*self))
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
        pub fn add_highlight(
            &self,
            lua: &Lua,
            ns_id: i32,
            hl_group: &str,
            line: usize,
            col_start: u32,
            col_end: i32,
        ) -> LuaResult<i32> {
            let lfn: LuaFunction = lua.load("vim.api.nvim_buf_add_highlight").eval()?;
            lfn.call((self.id(), ns_id, hl_group, line, col_start, col_end))
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
      • {start}            First line index
      • {end}              Last line index, exclusive
      • {strict_indexing}  Whether out-of-bounds should be an error.
      • {replacement}      Array of lines to use as replacement

    See also: ~
      • |nvim_buf_set_text()|
    */
        pub fn set_lines(
            &self,
            lua: &Lua,
            start: i32,
            end: i32,
            strict_indexing: bool,
            lines: &[String],
        ) -> mlua::Result<()> {
            let lfn: LuaFunction = lua.load("vim.api.nvim_buf_set_lines").eval()?;
            lfn.call((self.id(), start, end, strict_indexing, lines))
        }
        pub fn get_lines(
            &self,
            lua: &Lua,
            start: i32,
            end: i32,
            strict_indexing: bool,
        ) -> LuaResult<Vec<String>> {
            let lfn: LuaFunction = lua.load("vim.api.nvim_buf_get_lines").eval()?;
            lfn.call((self.id(), start, end, strict_indexing))
        }
        pub fn line_count(&self, lua: &Lua) -> LuaResult<usize> {
            let lfn: LuaFunction = lua.load("vim.api.nvim_buf_line_count").eval()?;
            lfn.call(self.id())
        }
        pub fn call<'a>(&self, lua: &'a Lua, cb: LuaFunction<'a>) -> LuaResult<()> {
            let lfn: LuaFunction = lua.load("vim.api.nvim_buf_call").eval()?;
            lfn.call((self.id(), cb))
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
      • {ns_id}   Namespace id from |nvim_create_namespace()|
      • {line}    Line where to place the mark, 0-based. |api-indexing|
      • {col}     Column where to place the mark, 0-based. |api-indexing|
      • {opts}    Optional parameters.
    */
        pub fn set_extmarks<'a>(
            &self,
            lua: &'a Lua,
            ns_id: u32,
            line: u32,
            col: u32,
            opts: ExtmarkOpts,
        ) -> LuaResult<()> {
            NeoApi::buf_set_extmark(lua, self.id(), ns_id, line, col, opts)
        }
        /**
    Clears |namespace|d objects (highlights, |extmarks|, virtual text) from a
    region.

    Lines are 0-indexed. |api-indexing| To clear the namespace in the entire
    buffer, specify line_start=0 and line_end=-1.

    Parameters: ~
      • {ns_id}       Namespace to clear, or -1 to clear all namespaces.
      • {line_start}  Start of range of lines to clear
      • {line_end}    End of range of lines to clear (exclusive) or -1 to
                      clear to end of buffer.
    */
        pub fn clear_namespace(
            &self,
            lua: &Lua,
            ns_id: i32,
            line_start: u32,
            line_end: i32,
        ) -> LuaResult<()> {
            let lfn: LuaFunction = lua.load("vim.api.nvim_buf_clear_namespace").eval()?;
            lfn.call((self.id(), ns_id, line_start, line_end))
        }
    }
}
mod callback {
    use crate::AutoCmdCbEvent;
    use mlua::Lua;
    use std::sync::OnceLock;
    use std::sync::Mutex;
    use mlua::prelude::LuaResult;
    pub const fn create_callback_container<T>() -> OnceLock<Mutex<CallBackQueue<T>>> {
        OnceLock::new()
    }
    pub trait InitCBQueue<T> {
        fn init(&self);
        fn push(&self, func: CbFunction<T>, ev: AutoCmdCbEvent) -> LuaResult<()>;
        fn exec(&self, state: &mut T, lua: &Lua) -> LuaResult<()>;
    }
    impl<T> InitCBQueue<T> for OnceLock<Mutex<CallBackQueue<T>>> {
        fn init(&self) {
            let _ = self.set(Mutex::new(CallBackQueue::default()));
        }
        fn push(&self, func: CbFunction<T>, ev: AutoCmdCbEvent) -> LuaResult<()> {
            let mut queue = self.get().unwrap().lock().unwrap();
            queue.push(func, ev);
            Ok(())
        }
        fn exec(&self, state: &mut T, lua: &Lua) -> LuaResult<()> {
            let mut queue = self.get().unwrap().lock().unwrap();
            queue.exec(state, lua);
            Ok(())
        }
    }
    pub type CbFunction<T> = Box<dyn Fn(&Lua, &mut T, AutoCmdCbEvent)>;
    pub struct CbArgs<T> {
        pub func: CbFunction<T>,
        pub ev: AutoCmdCbEvent,
    }
    /// Because autocmd callbacks are invoked before returning the NeoApi function calls
    /// It can deadlock your app, this makes sure a queue is added which can be called
    /// at the end of any module function implementation
    pub struct CallBackQueue<T>(Vec<CbArgs<T>>);
    impl<T> CallBackQueue<T> {
        fn push(&mut self, func: CbFunction<T>, ev: AutoCmdCbEvent) {
            self.0.push(CbArgs { ev, func });
        }
        fn exec(&mut self, state: &mut T, lua: &Lua) {
            while !self.0.is_empty() {
                let item = self.0.remove(0);
                (item.func)(lua, state, item.ev);
            }
        }
    }
    unsafe impl<T> Send for CallBackQueue<T> {}
    unsafe impl<T> Sync for CallBackQueue<T> {}
    impl<T> Default for CallBackQueue<T> {
        fn default() -> Self {
            Self(::alloc::vec::Vec::new())
        }
    }
}
mod neo_api {
    use crate::neo_api_types::{
        AutoCmd, AutoCmdEvent, AutoCmdOpts, ExtmarkOpts, LogLevel, Mode, OpenIn,
        OptValueType, StdpathType, Ui,
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
        pub fn delay<'a>(
            lua: &'a Lua,
            ms: u32,
            callback: LuaFunction<'a>,
        ) -> LuaResult<()> {
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
                .set(
                    {
                        let res = ::alloc::fmt::format(
                            format_args!("neo_timer_{0}", timer_id),
                        );
                        res
                    },
                    timer.clone(),
                )?;
            lfn.call((timer, ms, ms, callback))
        }
        pub fn stop_interval(lua: &Lua, timer_id: &str) -> LuaResult<()> {
            let timer: LuaValue = lua
                .globals()
                .get({
                    let res = ::alloc::fmt::format(
                        format_args!("neo_timer_{0}", timer_id),
                    );
                    res
                })?;
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
            lfn.call({
                let res = ::alloc::fmt::format(format_args!("{0:?}", debug));
                res
            })
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
        pub fn inspect<'lua, V: IntoLua<'lua>>(
            lua: &'lua Lua,
            table: V,
        ) -> LuaResult<()> {
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
            let cmd = {
                let res = ::alloc::fmt::format(format_args!("vim.cmd.{0}", opts.cmd));
                res
            };
            let lfn: LuaFunction = lua.load(cmd).eval()?;
            lfn.call((opts))
        }
        pub fn open_file(lua: &Lua, open_in: OpenIn, path: &str) -> LuaResult<()> {
            let lfn: LuaFunction = lua
                .load({
                    let res = ::alloc::fmt::format(format_args!("vim.cmd.{0}", open_in));
                    res
                })
                .eval()?;
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
}
mod neo_api_types {
    #![allow(unused)]
    use crate::buffer::NeoBuffer;
    use crate::{neo_api::NeoApi, window::NeoWindow};
    use macros::{FromTable, IntoEnum, IntoEnumSC, IntoTable};
    use mlua::prelude::*;
    use serde::{de::DeserializeOwned, Deserialize};
    use std::fmt::{self, Display};
    use std::ops;
    pub enum VirtTextPos {
        Eol,
        Overlay,
        RightAlign,
        Inline,
    }
    #[automatically_derived]
    impl ::core::fmt::Debug for VirtTextPos {
        #[inline]
        fn fmt(&self, f: &mut ::core::fmt::Formatter) -> ::core::fmt::Result {
            ::core::fmt::Formatter::write_str(
                f,
                match self {
                    VirtTextPos::Eol => "Eol",
                    VirtTextPos::Overlay => "Overlay",
                    VirtTextPos::RightAlign => "RightAlign",
                    VirtTextPos::Inline => "Inline",
                },
            )
        }
    }
    #[automatically_derived]
    impl ::core::clone::Clone for VirtTextPos {
        #[inline]
        fn clone(&self) -> VirtTextPos {
            *self
        }
    }
    #[automatically_derived]
    impl ::core::marker::Copy for VirtTextPos {}
    #[automatically_derived]
    impl ::core::marker::StructuralPartialEq for VirtTextPos {}
    #[automatically_derived]
    impl ::core::cmp::PartialEq for VirtTextPos {
        #[inline]
        fn eq(&self, other: &VirtTextPos) -> bool {
            let __self_tag = ::core::intrinsics::discriminant_value(self);
            let __arg1_tag = ::core::intrinsics::discriminant_value(other);
            __self_tag == __arg1_tag
        }
    }
    #[automatically_derived]
    impl ::core::cmp::Eq for VirtTextPos {
        #[inline]
        #[doc(hidden)]
        #[coverage(off)]
        fn assert_receiver_is_total_eq(&self) -> () {}
    }
    impl std::fmt::Display for VirtTextPos {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            match self {
                Self::Eol => f.write_str("eol"),
                Self::Overlay => f.write_str("overlay"),
                Self::RightAlign => f.write_str("right_align"),
                Self::Inline => f.write_str("inline"),
            }
        }
    }
    impl<'a> mlua::IntoLua<'a> for VirtTextPos {
        fn into_lua(self, lua: &'a Lua) -> mlua::Result<mlua::Value<'a>> {
            let str = lua.create_string(self.to_string())?;
            Ok(mlua::Value::String(str))
        }
    }
    pub enum HLMode {
        Replace,
        Combine,
        Blend,
    }
    #[automatically_derived]
    impl ::core::fmt::Debug for HLMode {
        #[inline]
        fn fmt(&self, f: &mut ::core::fmt::Formatter) -> ::core::fmt::Result {
            ::core::fmt::Formatter::write_str(
                f,
                match self {
                    HLMode::Replace => "Replace",
                    HLMode::Combine => "Combine",
                    HLMode::Blend => "Blend",
                },
            )
        }
    }
    #[automatically_derived]
    impl ::core::clone::Clone for HLMode {
        #[inline]
        fn clone(&self) -> HLMode {
            *self
        }
    }
    #[automatically_derived]
    impl ::core::marker::Copy for HLMode {}
    #[automatically_derived]
    impl ::core::marker::StructuralPartialEq for HLMode {}
    #[automatically_derived]
    impl ::core::cmp::PartialEq for HLMode {
        #[inline]
        fn eq(&self, other: &HLMode) -> bool {
            let __self_tag = ::core::intrinsics::discriminant_value(self);
            let __arg1_tag = ::core::intrinsics::discriminant_value(other);
            __self_tag == __arg1_tag
        }
    }
    #[automatically_derived]
    impl ::core::cmp::Eq for HLMode {
        #[inline]
        #[doc(hidden)]
        #[coverage(off)]
        fn assert_receiver_is_total_eq(&self) -> () {}
    }
    impl std::fmt::Display for HLMode {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            match self {
                Self::Replace => f.write_str("replace"),
                Self::Combine => f.write_str("combine"),
                Self::Blend => f.write_str("blend"),
            }
        }
    }
    impl<'a> mlua::IntoLua<'a> for HLMode {
        fn into_lua(self, lua: &'a Lua) -> mlua::Result<mlua::Value<'a>> {
            let str = lua.create_string(self.to_string())?;
            Ok(mlua::Value::String(str))
        }
    }
    pub struct HLText {
        pub text: String,
        pub highlight: String,
    }
    #[automatically_derived]
    impl ::core::fmt::Debug for HLText {
        #[inline]
        fn fmt(&self, f: &mut ::core::fmt::Formatter) -> ::core::fmt::Result {
            ::core::fmt::Formatter::debug_struct_field2_finish(
                f,
                "HLText",
                "text",
                &self.text,
                "highlight",
                &&self.highlight,
            )
        }
    }
    #[automatically_derived]
    impl ::core::clone::Clone for HLText {
        #[inline]
        fn clone(&self) -> HLText {
            HLText {
                text: ::core::clone::Clone::clone(&self.text),
                highlight: ::core::clone::Clone::clone(&self.highlight),
            }
        }
    }
    #[automatically_derived]
    impl ::core::marker::StructuralPartialEq for HLText {}
    #[automatically_derived]
    impl ::core::cmp::PartialEq for HLText {
        #[inline]
        fn eq(&self, other: &HLText) -> bool {
            self.text == other.text && self.highlight == other.highlight
        }
    }
    #[automatically_derived]
    impl ::core::cmp::Eq for HLText {
        #[inline]
        #[doc(hidden)]
        #[coverage(off)]
        fn assert_receiver_is_total_eq(&self) -> () {
            let _: ::core::cmp::AssertParamIsEq<String>;
        }
    }
    impl HLText {
        pub fn new<IntoStr: Into<String>>(text: IntoStr, highlight: IntoStr) -> Self {
            Self {
                text: text.into(),
                highlight: highlight.into(),
            }
        }
    }
    impl<'a> IntoLua<'a> for HLText {
        fn into_lua(self, lua: &'a Lua) -> LuaResult<LuaValue<'a>> {
            let table = lua.create_table()?;
            table.push(self.text)?;
            table.push(self.highlight)?;
            Ok(LuaValue::Table(table))
        }
    }
    pub trait ParseToLua<'a> {
        fn parse(self, lua: &'a Lua) -> LuaResult<LuaValue<'a>>;
    }
    impl<'a, T> ParseToLua<'a> for Vec<T>
    where
        T: IntoLua<'a>,
    {
        fn parse(self, lua: &'a Lua) -> LuaResult<LuaValue<'a>> {
            let out = lua.create_table()?;
            for tuple in self.into_iter() {
                out.push(tuple.into_lua(lua)?);
            }
            Ok(LuaValue::Table(out))
        }
    }
    pub enum TextType {
        String(String),
        Tuples(Vec<HLText>),
    }
    #[automatically_derived]
    impl ::core::fmt::Debug for TextType {
        #[inline]
        fn fmt(&self, f: &mut ::core::fmt::Formatter) -> ::core::fmt::Result {
            match self {
                TextType::String(__self_0) => {
                    ::core::fmt::Formatter::debug_tuple_field1_finish(
                        f,
                        "String",
                        &__self_0,
                    )
                }
                TextType::Tuples(__self_0) => {
                    ::core::fmt::Formatter::debug_tuple_field1_finish(
                        f,
                        "Tuples",
                        &__self_0,
                    )
                }
            }
        }
    }
    #[automatically_derived]
    impl ::core::clone::Clone for TextType {
        #[inline]
        fn clone(&self) -> TextType {
            match self {
                TextType::String(__self_0) => {
                    TextType::String(::core::clone::Clone::clone(__self_0))
                }
                TextType::Tuples(__self_0) => {
                    TextType::Tuples(::core::clone::Clone::clone(__self_0))
                }
            }
        }
    }
    #[automatically_derived]
    impl ::core::marker::StructuralPartialEq for TextType {}
    #[automatically_derived]
    impl ::core::cmp::PartialEq for TextType {
        #[inline]
        fn eq(&self, other: &TextType) -> bool {
            let __self_tag = ::core::intrinsics::discriminant_value(self);
            let __arg1_tag = ::core::intrinsics::discriminant_value(other);
            __self_tag == __arg1_tag
                && match (self, other) {
                    (TextType::String(__self_0), TextType::String(__arg1_0)) => {
                        *__self_0 == *__arg1_0
                    }
                    (TextType::Tuples(__self_0), TextType::Tuples(__arg1_0)) => {
                        *__self_0 == *__arg1_0
                    }
                    _ => unsafe { ::core::intrinsics::unreachable() }
                }
        }
    }
    #[automatically_derived]
    impl ::core::cmp::Eq for TextType {
        #[inline]
        #[doc(hidden)]
        #[coverage(off)]
        fn assert_receiver_is_total_eq(&self) -> () {
            let _: ::core::cmp::AssertParamIsEq<String>;
            let _: ::core::cmp::AssertParamIsEq<Vec<HLText>>;
        }
    }
    impl<'a> IntoLua<'a> for TextType {
        fn into_lua(self, lua: &'a Lua) -> LuaResult<LuaValue<'a>> {
            match self {
                Self::String(str) => Ok(LuaValue::String(lua.create_string(str)?)),
                Self::Tuples(tuples) => tuples.into_lua(lua),
            }
        }
    }
    pub enum OptValueType {
        Window(NeoWindow),
        Buffer(NeoBuffer),
    }
    pub enum LogLevel {
        Trace = 0,
        Debug = 1,
        Info = 2,
        Warn = 3,
        Error = 4,
        Off = 5,
    }
    #[automatically_derived]
    impl ::core::fmt::Debug for LogLevel {
        #[inline]
        fn fmt(&self, f: &mut ::core::fmt::Formatter) -> ::core::fmt::Result {
            ::core::fmt::Formatter::write_str(
                f,
                match self {
                    LogLevel::Trace => "Trace",
                    LogLevel::Debug => "Debug",
                    LogLevel::Info => "Info",
                    LogLevel::Warn => "Warn",
                    LogLevel::Error => "Error",
                    LogLevel::Off => "Off",
                },
            )
        }
    }
    #[automatically_derived]
    impl ::core::clone::Clone for LogLevel {
        #[inline]
        fn clone(&self) -> LogLevel {
            *self
        }
    }
    #[automatically_derived]
    impl ::core::marker::Copy for LogLevel {}
    #[automatically_derived]
    impl ::core::marker::StructuralPartialEq for LogLevel {}
    #[automatically_derived]
    impl ::core::cmp::PartialEq for LogLevel {
        #[inline]
        fn eq(&self, other: &LogLevel) -> bool {
            let __self_tag = ::core::intrinsics::discriminant_value(self);
            let __arg1_tag = ::core::intrinsics::discriminant_value(other);
            __self_tag == __arg1_tag
        }
    }
    #[automatically_derived]
    impl ::core::cmp::Eq for LogLevel {
        #[inline]
        #[doc(hidden)]
        #[coverage(off)]
        fn assert_receiver_is_total_eq(&self) -> () {}
    }
    pub enum StdpathType {
        /// Cache directory: arbitrary temporary storage for plugins, etc.
        Cache,
        /// User configuration directory. |init.vim| is stored here.
        Config,
        /// User data directory.
        Data,
        /// Logs directory (for use by plugins too).
        Log,
        /// Run directory: temporary, local storage for sockets, named pipes, etc.
        Run,
        /// Session state directory: storage for file drafts, swap, undo, |shada|.
        State,
    }
    #[automatically_derived]
    impl ::core::clone::Clone for StdpathType {
        #[inline]
        fn clone(&self) -> StdpathType {
            *self
        }
    }
    #[automatically_derived]
    impl ::core::marker::Copy for StdpathType {}
    #[automatically_derived]
    impl ::core::marker::StructuralPartialEq for StdpathType {}
    #[automatically_derived]
    impl ::core::cmp::PartialEq for StdpathType {
        #[inline]
        fn eq(&self, other: &StdpathType) -> bool {
            let __self_tag = ::core::intrinsics::discriminant_value(self);
            let __arg1_tag = ::core::intrinsics::discriminant_value(other);
            __self_tag == __arg1_tag
        }
    }
    #[automatically_derived]
    impl ::core::cmp::Eq for StdpathType {
        #[inline]
        #[doc(hidden)]
        #[coverage(off)]
        fn assert_receiver_is_total_eq(&self) -> () {}
    }
    impl std::fmt::Display for StdpathType {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            match self {
                Self::Cache => f.write_str("cache"),
                Self::Config => f.write_str("config"),
                Self::Data => f.write_str("data"),
                Self::Log => f.write_str("log"),
                Self::Run => f.write_str("run"),
                Self::State => f.write_str("state"),
            }
        }
    }
    impl<'a> mlua::IntoLua<'a> for StdpathType {
        fn into_lua(self, lua: &'a Lua) -> mlua::Result<mlua::Value<'a>> {
            let str = lua.create_string(self.to_string())?;
            Ok(mlua::Value::String(str))
        }
    }
    pub enum OpenIn {
        Buffer,
        VSplit,
        HSplit,
        Tab,
    }
    #[automatically_derived]
    impl ::core::clone::Clone for OpenIn {
        #[inline]
        fn clone(&self) -> OpenIn {
            *self
        }
    }
    #[automatically_derived]
    impl ::core::marker::Copy for OpenIn {}
    #[automatically_derived]
    impl ::core::fmt::Debug for OpenIn {
        #[inline]
        fn fmt(&self, f: &mut ::core::fmt::Formatter) -> ::core::fmt::Result {
            ::core::fmt::Formatter::write_str(
                f,
                match self {
                    OpenIn::Buffer => "Buffer",
                    OpenIn::VSplit => "VSplit",
                    OpenIn::HSplit => "HSplit",
                    OpenIn::Tab => "Tab",
                },
            )
        }
    }
    #[automatically_derived]
    impl ::core::marker::StructuralPartialEq for OpenIn {}
    #[automatically_derived]
    impl ::core::cmp::PartialEq for OpenIn {
        #[inline]
        fn eq(&self, other: &OpenIn) -> bool {
            let __self_tag = ::core::intrinsics::discriminant_value(self);
            let __arg1_tag = ::core::intrinsics::discriminant_value(other);
            __self_tag == __arg1_tag
        }
    }
    impl Display for OpenIn {
        fn fmt(&self, f: &mut fmt::Formatter<'_>) -> std::fmt::Result {
            match self {
                Self::Buffer => f.write_str("edit"),
                Self::Tab => f.write_str("tabedit"),
                Self::VSplit => f.write_str("vsplit"),
                Self::HSplit => f.write_str("split"),
            }
        }
    }
    /// Pleas help to add more and test
    pub struct ExtmarkOpts {
        /// id : id of the extmark to edit.
        pub id: Option<u32>,
        /// end_row : ending line of the mark, 0-based inclusive.
        pub end_row: Option<i32>,
        /// end_col : ending col of the mark, 0-based exclusive.
        pub end_col: Option<i32>,
        /// hl_eol : when true, for a multiline highlight covering the
        /// EOL of a line, continue the highlight for the rest of the
        /// screen line (just like for diff and cursorline highlight).
        pub hl_eol: Option<bool>,
        /// hl_group : name of the highlight group used to highlight this mark.
        pub hl_group: Option<String>,
        /// hl_mode : control how highlights are combined with the
        /// highlights of the text. Currently only affects virt_text
        /// highlights, but might affect `hl_group` in later versions.
        /// • "replace": only show the virt_text color. This is the
        ///   default.
        /// • "combine": combine with background text color.
        /// • "blend": blend with background text color. Not supported
        ///   for "inline" virt_text.
        pub hl_mode: Option<HLMode>,
        /// virtual text to link to this mark. A list of
        /// `[text, highlight]` tuples, each representing a text chunk
        /// with specified highlight. `highlight` element can either
        /// be a single highlight group, or an array of multiple
        /// highlight groups that will be stacked (highest priority
        /// last). A highlight group can be supplied either as a
        /// string or as an integer, the latter which can be obtained
        /// using |nvim_get_hl_id_by_name()|.
        pub virt_text: Option<Vec<HLText>>,
        /// virt_text_pos : position of virtual text. Possible values:
        /// • "eol": right after eol character (default).
        /// • "overlay": display over the specified column, without
        ///   shifting the underlying text.
        /// • "right_align": display right aligned in the window.
        /// • "inline": display at the specified column, and shift the
        ///   buffer text to the right as needed.
        pub virt_text_pos: Option<VirtTextPos>,
        /// virt_text_win_col : position the virtual text at a fixed
        /// window column (starting from the first text column of the
        /// screen line) instead of "virt_text_pos".
        /// virt_text_hide : hide the virtual text when the background
        /// text is selected or hidden because of scrolling with
        /// 'nowrap' or 'smoothscroll'. Currently only affects
        /// "overlay" virt_text.
        pub virt_text_win_col: Option<u32>,
        /// virt_text_repeat_linebreak : repeat the virtual text on
        /// wrapped lines.
        pub virt_text_repeat_linebreak: Option<bool>,
        /// virt_lines_above: place virtual lines above instead.
        pub virt_lines_above: Option<bool>,
    }
    #[automatically_derived]
    impl ::core::fmt::Debug for ExtmarkOpts {
        #[inline]
        fn fmt(&self, f: &mut ::core::fmt::Formatter) -> ::core::fmt::Result {
            let names: &'static _ = &[
                "id",
                "end_row",
                "end_col",
                "hl_eol",
                "hl_group",
                "hl_mode",
                "virt_text",
                "virt_text_pos",
                "virt_text_win_col",
                "virt_text_repeat_linebreak",
                "virt_lines_above",
            ];
            let values: &[&dyn ::core::fmt::Debug] = &[
                &self.id,
                &self.end_row,
                &self.end_col,
                &self.hl_eol,
                &self.hl_group,
                &self.hl_mode,
                &self.virt_text,
                &self.virt_text_pos,
                &self.virt_text_win_col,
                &self.virt_text_repeat_linebreak,
                &&self.virt_lines_above,
            ];
            ::core::fmt::Formatter::debug_struct_fields_finish(
                f,
                "ExtmarkOpts",
                names,
                values,
            )
        }
    }
    #[automatically_derived]
    impl ::core::default::Default for ExtmarkOpts {
        #[inline]
        fn default() -> ExtmarkOpts {
            ExtmarkOpts {
                id: ::core::default::Default::default(),
                end_row: ::core::default::Default::default(),
                end_col: ::core::default::Default::default(),
                hl_eol: ::core::default::Default::default(),
                hl_group: ::core::default::Default::default(),
                hl_mode: ::core::default::Default::default(),
                virt_text: ::core::default::Default::default(),
                virt_text_pos: ::core::default::Default::default(),
                virt_text_win_col: ::core::default::Default::default(),
                virt_text_repeat_linebreak: ::core::default::Default::default(),
                virt_lines_above: ::core::default::Default::default(),
            }
        }
    }
    #[automatically_derived]
    impl ::core::clone::Clone for ExtmarkOpts {
        #[inline]
        fn clone(&self) -> ExtmarkOpts {
            ExtmarkOpts {
                id: ::core::clone::Clone::clone(&self.id),
                end_row: ::core::clone::Clone::clone(&self.end_row),
                end_col: ::core::clone::Clone::clone(&self.end_col),
                hl_eol: ::core::clone::Clone::clone(&self.hl_eol),
                hl_group: ::core::clone::Clone::clone(&self.hl_group),
                hl_mode: ::core::clone::Clone::clone(&self.hl_mode),
                virt_text: ::core::clone::Clone::clone(&self.virt_text),
                virt_text_pos: ::core::clone::Clone::clone(&self.virt_text_pos),
                virt_text_win_col: ::core::clone::Clone::clone(&self.virt_text_win_col),
                virt_text_repeat_linebreak: ::core::clone::Clone::clone(
                    &self.virt_text_repeat_linebreak,
                ),
                virt_lines_above: ::core::clone::Clone::clone(&self.virt_lines_above),
            }
        }
    }
    impl<'a> mlua::IntoLua<'a> for ExtmarkOpts {
        fn into_lua(self, lua: &'a Lua) -> mlua::Result<mlua::Value<'a>> {
            let out = lua.create_table()?;
            if let Some(value) = self.id {
                out.set("id", value)?;
            }
            if let Some(value) = self.end_row {
                out.set("end_row", value)?;
            }
            if let Some(value) = self.end_col {
                out.set("end_col", value)?;
            }
            if let Some(value) = self.hl_eol {
                out.set("hl_eol", value)?;
            }
            if let Some(value) = self.hl_group {
                out.set("hl_group", value)?;
            }
            if let Some(value) = self.hl_mode {
                out.set("hl_mode", value)?;
            }
            if let Some(value) = self.virt_text {
                out.set("virt_text", value)?;
            }
            if let Some(value) = self.virt_text_pos {
                out.set("virt_text_pos", value)?;
            }
            if let Some(value) = self.virt_text_win_col {
                out.set("virt_text_win_col", value)?;
            }
            if let Some(value) = self.virt_text_repeat_linebreak {
                out.set("virt_text_repeat_linebreak", value)?;
            }
            if let Some(value) = self.virt_lines_above {
                out.set("virt_lines_above", value)?;
            }
            Ok(mlua::Value::Table(out))
        }
    }
    pub struct KeymapOpts {
        pub silent: Option<bool>,
        pub buffer: Option<u32>,
    }
    #[automatically_derived]
    impl ::core::fmt::Debug for KeymapOpts {
        #[inline]
        fn fmt(&self, f: &mut ::core::fmt::Formatter) -> ::core::fmt::Result {
            ::core::fmt::Formatter::debug_struct_field2_finish(
                f,
                "KeymapOpts",
                "silent",
                &self.silent,
                "buffer",
                &&self.buffer,
            )
        }
    }
    #[automatically_derived]
    impl ::core::default::Default for KeymapOpts {
        #[inline]
        fn default() -> KeymapOpts {
            KeymapOpts {
                silent: ::core::default::Default::default(),
                buffer: ::core::default::Default::default(),
            }
        }
    }
    impl<'a> mlua::IntoLua<'a> for KeymapOpts {
        fn into_lua(self, lua: &'a Lua) -> mlua::Result<mlua::Value<'a>> {
            let out = lua.create_table()?;
            if let Some(value) = self.silent {
                out.set("silent", value)?;
            }
            if let Some(value) = self.buffer {
                out.set("buffer", value)?;
            }
            Ok(mlua::Value::Table(out))
        }
    }
    #[automatically_derived]
    impl ::core::clone::Clone for KeymapOpts {
        #[inline]
        fn clone(&self) -> KeymapOpts {
            let _: ::core::clone::AssertParamIsClone<Option<bool>>;
            let _: ::core::clone::AssertParamIsClone<Option<u32>>;
            *self
        }
    }
    #[automatically_derived]
    impl ::core::marker::Copy for KeymapOpts {}
    impl KeymapOpts {
        pub fn new(buf_id: u32, silent: bool) -> Self {
            Self {
                buffer: Some(buf_id),
                silent: Some(silent),
            }
        }
    }
    pub struct BufferDeleteOpts {
        pub force: bool,
        pub unload: bool,
    }
    #[automatically_derived]
    impl ::core::fmt::Debug for BufferDeleteOpts {
        #[inline]
        fn fmt(&self, f: &mut ::core::fmt::Formatter) -> ::core::fmt::Result {
            ::core::fmt::Formatter::debug_struct_field2_finish(
                f,
                "BufferDeleteOpts",
                "force",
                &self.force,
                "unload",
                &&self.unload,
            )
        }
    }
    #[automatically_derived]
    impl ::core::clone::Clone for BufferDeleteOpts {
        #[inline]
        fn clone(&self) -> BufferDeleteOpts {
            let _: ::core::clone::AssertParamIsClone<bool>;
            *self
        }
    }
    #[automatically_derived]
    impl ::core::marker::Copy for BufferDeleteOpts {}
    impl<'a> mlua::IntoLua<'a> for BufferDeleteOpts {
        fn into_lua(self, lua: &'a Lua) -> mlua::Result<mlua::Value<'a>> {
            let out = lua.create_table()?;
            out.set("force", self.force)?;
            out.set("unload", self.unload)?;
            Ok(mlua::Value::Table(out))
        }
    }
    #[automatically_derived]
    impl ::core::default::Default for BufferDeleteOpts {
        #[inline]
        fn default() -> BufferDeleteOpts {
            BufferDeleteOpts {
                force: ::core::default::Default::default(),
                unload: ::core::default::Default::default(),
            }
        }
    }
    pub struct WinCursor {
        row: u32,
        pub column: u32,
    }
    #[automatically_derived]
    impl ::core::fmt::Debug for WinCursor {
        #[inline]
        fn fmt(&self, f: &mut ::core::fmt::Formatter) -> ::core::fmt::Result {
            ::core::fmt::Formatter::debug_struct_field2_finish(
                f,
                "WinCursor",
                "row",
                &self.row,
                "column",
                &&self.column,
            )
        }
    }
    #[doc(hidden)]
    #[allow(non_upper_case_globals, unused_attributes, unused_qualifications)]
    const _: () = {
        #[allow(unused_extern_crates, clippy::useless_attribute)]
        extern crate serde as _serde;
        #[automatically_derived]
        impl<'de> _serde::Deserialize<'de> for WinCursor {
            fn deserialize<__D>(
                __deserializer: __D,
            ) -> _serde::__private::Result<Self, __D::Error>
            where
                __D: _serde::Deserializer<'de>,
            {
                #[allow(non_camel_case_types)]
                #[doc(hidden)]
                enum __Field {
                    __field0,
                    __field1,
                    __ignore,
                }
                #[doc(hidden)]
                struct __FieldVisitor;
                impl<'de> _serde::de::Visitor<'de> for __FieldVisitor {
                    type Value = __Field;
                    fn expecting(
                        &self,
                        __formatter: &mut _serde::__private::Formatter,
                    ) -> _serde::__private::fmt::Result {
                        _serde::__private::Formatter::write_str(
                            __formatter,
                            "field identifier",
                        )
                    }
                    fn visit_u64<__E>(
                        self,
                        __value: u64,
                    ) -> _serde::__private::Result<Self::Value, __E>
                    where
                        __E: _serde::de::Error,
                    {
                        match __value {
                            0u64 => _serde::__private::Ok(__Field::__field0),
                            1u64 => _serde::__private::Ok(__Field::__field1),
                            _ => _serde::__private::Ok(__Field::__ignore),
                        }
                    }
                    fn visit_str<__E>(
                        self,
                        __value: &str,
                    ) -> _serde::__private::Result<Self::Value, __E>
                    where
                        __E: _serde::de::Error,
                    {
                        match __value {
                            "row" => _serde::__private::Ok(__Field::__field0),
                            "column" => _serde::__private::Ok(__Field::__field1),
                            _ => _serde::__private::Ok(__Field::__ignore),
                        }
                    }
                    fn visit_bytes<__E>(
                        self,
                        __value: &[u8],
                    ) -> _serde::__private::Result<Self::Value, __E>
                    where
                        __E: _serde::de::Error,
                    {
                        match __value {
                            b"row" => _serde::__private::Ok(__Field::__field0),
                            b"column" => _serde::__private::Ok(__Field::__field1),
                            _ => _serde::__private::Ok(__Field::__ignore),
                        }
                    }
                }
                impl<'de> _serde::Deserialize<'de> for __Field {
                    #[inline]
                    fn deserialize<__D>(
                        __deserializer: __D,
                    ) -> _serde::__private::Result<Self, __D::Error>
                    where
                        __D: _serde::Deserializer<'de>,
                    {
                        _serde::Deserializer::deserialize_identifier(
                            __deserializer,
                            __FieldVisitor,
                        )
                    }
                }
                #[doc(hidden)]
                struct __Visitor<'de> {
                    marker: _serde::__private::PhantomData<WinCursor>,
                    lifetime: _serde::__private::PhantomData<&'de ()>,
                }
                impl<'de> _serde::de::Visitor<'de> for __Visitor<'de> {
                    type Value = WinCursor;
                    fn expecting(
                        &self,
                        __formatter: &mut _serde::__private::Formatter,
                    ) -> _serde::__private::fmt::Result {
                        _serde::__private::Formatter::write_str(
                            __formatter,
                            "struct WinCursor",
                        )
                    }
                    #[inline]
                    fn visit_seq<__A>(
                        self,
                        mut __seq: __A,
                    ) -> _serde::__private::Result<Self::Value, __A::Error>
                    where
                        __A: _serde::de::SeqAccess<'de>,
                    {
                        let __field0 = match _serde::de::SeqAccess::next_element::<
                            u32,
                        >(&mut __seq)? {
                            _serde::__private::Some(__value) => __value,
                            _serde::__private::None => {
                                return _serde::__private::Err(
                                    _serde::de::Error::invalid_length(
                                        0usize,
                                        &"struct WinCursor with 2 elements",
                                    ),
                                );
                            }
                        };
                        let __field1 = match _serde::de::SeqAccess::next_element::<
                            u32,
                        >(&mut __seq)? {
                            _serde::__private::Some(__value) => __value,
                            _serde::__private::None => {
                                return _serde::__private::Err(
                                    _serde::de::Error::invalid_length(
                                        1usize,
                                        &"struct WinCursor with 2 elements",
                                    ),
                                );
                            }
                        };
                        _serde::__private::Ok(WinCursor {
                            row: __field0,
                            column: __field1,
                        })
                    }
                    #[inline]
                    fn visit_map<__A>(
                        self,
                        mut __map: __A,
                    ) -> _serde::__private::Result<Self::Value, __A::Error>
                    where
                        __A: _serde::de::MapAccess<'de>,
                    {
                        let mut __field0: _serde::__private::Option<u32> = _serde::__private::None;
                        let mut __field1: _serde::__private::Option<u32> = _serde::__private::None;
                        while let _serde::__private::Some(__key) = _serde::de::MapAccess::next_key::<
                            __Field,
                        >(&mut __map)? {
                            match __key {
                                __Field::__field0 => {
                                    if _serde::__private::Option::is_some(&__field0) {
                                        return _serde::__private::Err(
                                            <__A::Error as _serde::de::Error>::duplicate_field("row"),
                                        );
                                    }
                                    __field0 = _serde::__private::Some(
                                        _serde::de::MapAccess::next_value::<u32>(&mut __map)?,
                                    );
                                }
                                __Field::__field1 => {
                                    if _serde::__private::Option::is_some(&__field1) {
                                        return _serde::__private::Err(
                                            <__A::Error as _serde::de::Error>::duplicate_field("column"),
                                        );
                                    }
                                    __field1 = _serde::__private::Some(
                                        _serde::de::MapAccess::next_value::<u32>(&mut __map)?,
                                    );
                                }
                                _ => {
                                    let _ = _serde::de::MapAccess::next_value::<
                                        _serde::de::IgnoredAny,
                                    >(&mut __map)?;
                                }
                            }
                        }
                        let __field0 = match __field0 {
                            _serde::__private::Some(__field0) => __field0,
                            _serde::__private::None => {
                                _serde::__private::de::missing_field("row")?
                            }
                        };
                        let __field1 = match __field1 {
                            _serde::__private::Some(__field1) => __field1,
                            _serde::__private::None => {
                                _serde::__private::de::missing_field("column")?
                            }
                        };
                        _serde::__private::Ok(WinCursor {
                            row: __field0,
                            column: __field1,
                        })
                    }
                }
                #[doc(hidden)]
                const FIELDS: &'static [&'static str] = &["row", "column"];
                _serde::Deserializer::deserialize_struct(
                    __deserializer,
                    "WinCursor",
                    FIELDS,
                    __Visitor {
                        marker: _serde::__private::PhantomData::<WinCursor>,
                        lifetime: _serde::__private::PhantomData,
                    },
                )
            }
        }
    };
    #[automatically_derived]
    impl ::core::clone::Clone for WinCursor {
        #[inline]
        fn clone(&self) -> WinCursor {
            let _: ::core::clone::AssertParamIsClone<u32>;
            *self
        }
    }
    #[automatically_derived]
    impl ::core::marker::Copy for WinCursor {}
    impl WinCursor {
        /// Create a cursor where the passed row argument starts from 0
        pub fn from_zero_indexed(row: u32, column: u32) -> Self {
            Self { row: row + 1, column }
        }
        /// Create a cursor where the passed row argument starts from 1
        pub fn from_one_indexed(row: u32, column: u32) -> Self {
            Self { row, column }
        }
        /// Return value starts from 0
        pub fn row_zero_indexed(&self) -> u32 {
            self.row - 1
        }
        /// Return value starts from 1
        pub fn row_one_indexed(&self) -> u32 {
            self.row
        }
    }
    impl<'a> FromLua<'a> for WinCursor {
        fn from_lua(value: LuaValue<'a>, lua: &'a Lua) -> LuaResult<Self> {
            match value {
                LuaValue::Table(table) => {
                    let row: u32 = table.get(1)?;
                    let column: u32 = table.get(2)?;
                    Ok(Self { row, column })
                }
                _ => {
                    Err(LuaError::DeserializeError("Supposed to be a table".to_string()))
                }
            }
        }
    }
    impl<'a> IntoLua<'a> for WinCursor {
        fn into_lua(self, lua: &'a Lua) -> LuaResult<LuaValue<'a>> {
            let table = lua.create_table()?;
            table.set(1, self.row);
            table.set(2, self.column);
            Ok(LuaValue::Table(table))
        }
    }
    pub struct Ui {
        pub chan: u32,
        pub ext_cmdline: bool,
        pub ext_hlstate: bool,
        pub ext_linegrid: bool,
        pub ext_messages: bool,
        pub ext_multigrid: bool,
        pub ext_popupmenu: bool,
        pub ext_tabline: bool,
        pub ext_termcolors: bool,
        pub ext_wildmenu: bool,
        pub height: u32,
        pub r#override: bool,
        pub rgb: bool,
        pub stdin_tty: bool,
        pub stdout_tty: bool,
        pub term_background: String,
        pub term_colors: u32,
        pub term_name: String,
        pub width: u32,
    }
    #[automatically_derived]
    impl ::core::fmt::Debug for Ui {
        #[inline]
        fn fmt(&self, f: &mut ::core::fmt::Formatter) -> ::core::fmt::Result {
            let names: &'static _ = &[
                "chan",
                "ext_cmdline",
                "ext_hlstate",
                "ext_linegrid",
                "ext_messages",
                "ext_multigrid",
                "ext_popupmenu",
                "ext_tabline",
                "ext_termcolors",
                "ext_wildmenu",
                "height",
                "override",
                "rgb",
                "stdin_tty",
                "stdout_tty",
                "term_background",
                "term_colors",
                "term_name",
                "width",
            ];
            let values: &[&dyn ::core::fmt::Debug] = &[
                &self.chan,
                &self.ext_cmdline,
                &self.ext_hlstate,
                &self.ext_linegrid,
                &self.ext_messages,
                &self.ext_multigrid,
                &self.ext_popupmenu,
                &self.ext_tabline,
                &self.ext_termcolors,
                &self.ext_wildmenu,
                &self.height,
                &self.r#override,
                &self.rgb,
                &self.stdin_tty,
                &self.stdout_tty,
                &self.term_background,
                &self.term_colors,
                &self.term_name,
                &&self.width,
            ];
            ::core::fmt::Formatter::debug_struct_fields_finish(f, "Ui", names, values)
        }
    }
    impl<'a> mlua::FromLua<'a> for Ui {
        fn from_lua(value: mlua::Value<'a>, lua: &'a Lua) -> mlua::Result<Self> {
            if let mlua::Value::Table(table) = value {
                Ok(Self {
                    chan: table.get("chan")?,
                    ext_cmdline: table.get("ext_cmdline")?,
                    ext_hlstate: table.get("ext_hlstate")?,
                    ext_linegrid: table.get("ext_linegrid")?,
                    ext_messages: table.get("ext_messages")?,
                    ext_multigrid: table.get("ext_multigrid")?,
                    ext_popupmenu: table.get("ext_popupmenu")?,
                    ext_tabline: table.get("ext_tabline")?,
                    ext_termcolors: table.get("ext_termcolors")?,
                    ext_wildmenu: table.get("ext_wildmenu")?,
                    height: table.get("height")?,
                    r#override: table.get("override")?,
                    rgb: table.get("rgb")?,
                    stdin_tty: table.get("stdin_tty")?,
                    stdout_tty: table.get("stdout_tty")?,
                    term_background: table.get("term_background")?,
                    term_colors: table.get("term_colors")?,
                    term_name: table.get("term_name")?,
                    width: table.get("width")?,
                })
            } else {
                Err(mlua::Error::FromLuaConversionError {
                    from: value.type_name(),
                    to: "Ui",
                    message: None,
                })
            }
        }
    }
    #[automatically_derived]
    impl ::core::clone::Clone for Ui {
        #[inline]
        fn clone(&self) -> Ui {
            Ui {
                chan: ::core::clone::Clone::clone(&self.chan),
                ext_cmdline: ::core::clone::Clone::clone(&self.ext_cmdline),
                ext_hlstate: ::core::clone::Clone::clone(&self.ext_hlstate),
                ext_linegrid: ::core::clone::Clone::clone(&self.ext_linegrid),
                ext_messages: ::core::clone::Clone::clone(&self.ext_messages),
                ext_multigrid: ::core::clone::Clone::clone(&self.ext_multigrid),
                ext_popupmenu: ::core::clone::Clone::clone(&self.ext_popupmenu),
                ext_tabline: ::core::clone::Clone::clone(&self.ext_tabline),
                ext_termcolors: ::core::clone::Clone::clone(&self.ext_termcolors),
                ext_wildmenu: ::core::clone::Clone::clone(&self.ext_wildmenu),
                height: ::core::clone::Clone::clone(&self.height),
                r#override: ::core::clone::Clone::clone(&self.r#override),
                rgb: ::core::clone::Clone::clone(&self.rgb),
                stdin_tty: ::core::clone::Clone::clone(&self.stdin_tty),
                stdout_tty: ::core::clone::Clone::clone(&self.stdout_tty),
                term_background: ::core::clone::Clone::clone(&self.term_background),
                term_colors: ::core::clone::Clone::clone(&self.term_colors),
                term_name: ::core::clone::Clone::clone(&self.term_name),
                width: ::core::clone::Clone::clone(&self.width),
            }
        }
    }
    pub enum Mode {
        Normal,
        Insert,
        Visual,
        Select,
    }
    #[automatically_derived]
    impl ::core::clone::Clone for Mode {
        #[inline]
        fn clone(&self) -> Mode {
            *self
        }
    }
    #[automatically_derived]
    impl ::core::marker::Copy for Mode {}
    #[automatically_derived]
    impl ::core::fmt::Debug for Mode {
        #[inline]
        fn fmt(&self, f: &mut ::core::fmt::Formatter) -> ::core::fmt::Result {
            ::core::fmt::Formatter::write_str(
                f,
                match self {
                    Mode::Normal => "Normal",
                    Mode::Insert => "Insert",
                    Mode::Visual => "Visual",
                    Mode::Select => "Select",
                },
            )
        }
    }
    #[automatically_derived]
    impl ::core::marker::StructuralPartialEq for Mode {}
    #[automatically_derived]
    impl ::core::cmp::PartialEq for Mode {
        #[inline]
        fn eq(&self, other: &Mode) -> bool {
            let __self_tag = ::core::intrinsics::discriminant_value(self);
            let __arg1_tag = ::core::intrinsics::discriminant_value(other);
            __self_tag == __arg1_tag
        }
    }
    #[automatically_derived]
    impl ::core::cmp::Eq for Mode {
        #[inline]
        #[doc(hidden)]
        #[coverage(off)]
        fn assert_receiver_is_total_eq(&self) -> () {}
    }
    impl Mode {
        pub fn get_char(&self) -> char {
            match self {
                Mode::Insert => 'i',
                Mode::Normal => 'n',
                Mode::Visual => 'v',
                Mode::Select => 's',
            }
        }
        pub fn get_str(&self) -> &str {
            match self {
                Mode::Insert => "i",
                Mode::Normal => "n",
                Mode::Visual => "v",
                Mode::Select => "s",
            }
        }
    }
    pub struct AutoCmd(u32);
    #[automatically_derived]
    impl ::core::clone::Clone for AutoCmd {
        #[inline]
        fn clone(&self) -> AutoCmd {
            let _: ::core::clone::AssertParamIsClone<u32>;
            *self
        }
    }
    #[automatically_derived]
    impl ::core::marker::Copy for AutoCmd {}
    #[automatically_derived]
    impl ::core::fmt::Debug for AutoCmd {
        #[inline]
        fn fmt(&self, f: &mut ::core::fmt::Formatter) -> ::core::fmt::Result {
            ::core::fmt::Formatter::debug_tuple_field1_finish(f, "AutoCmd", &&self.0)
        }
    }
    #[automatically_derived]
    impl ::core::marker::StructuralPartialEq for AutoCmd {}
    #[automatically_derived]
    impl ::core::cmp::PartialEq for AutoCmd {
        #[inline]
        fn eq(&self, other: &AutoCmd) -> bool {
            self.0 == other.0
        }
    }
    #[automatically_derived]
    impl ::core::cmp::Eq for AutoCmd {
        #[inline]
        #[doc(hidden)]
        #[coverage(off)]
        fn assert_receiver_is_total_eq(&self) -> () {
            let _: ::core::cmp::AssertParamIsEq<u32>;
        }
    }
    impl AutoCmd {
        pub fn new(id: u32) -> Self {
            Self(id)
        }
        pub fn id(&self) -> u32 {
            self.0
        }
    }
    pub enum AutoCmdGroup {
        String(String),
        Integer(u32),
    }
    #[automatically_derived]
    impl ::core::clone::Clone for AutoCmdGroup {
        #[inline]
        fn clone(&self) -> AutoCmdGroup {
            match self {
                AutoCmdGroup::String(__self_0) => {
                    AutoCmdGroup::String(::core::clone::Clone::clone(__self_0))
                }
                AutoCmdGroup::Integer(__self_0) => {
                    AutoCmdGroup::Integer(::core::clone::Clone::clone(__self_0))
                }
            }
        }
    }
    #[automatically_derived]
    impl ::core::fmt::Debug for AutoCmdGroup {
        #[inline]
        fn fmt(&self, f: &mut ::core::fmt::Formatter) -> ::core::fmt::Result {
            match self {
                AutoCmdGroup::String(__self_0) => {
                    ::core::fmt::Formatter::debug_tuple_field1_finish(
                        f,
                        "String",
                        &__self_0,
                    )
                }
                AutoCmdGroup::Integer(__self_0) => {
                    ::core::fmt::Formatter::debug_tuple_field1_finish(
                        f,
                        "Integer",
                        &__self_0,
                    )
                }
            }
        }
    }
    pub enum AutoCmdEvent {
        /// After adding a buffer to the buffer list.
        BufAdd,
        /// Deleting a buffer from the buffer list.
        BufDelete,
        /// After entering a buffer.
        BufEnter,
        /// After renaming a buffer.
        BufFilePost,
        /// Before renaming a buffer.
        BufFilePre,
        /// Just after buffer becomes hidden.
        BufHidden,
        /// Before leaving a buffer.
        BufLeave,
        /// After the 'modified' state of a buffer changes.
        BufModifiedSet,
        /// After creating any buffer.
        BufNew,
        /// When creating a buffer for a new file.
        BufNewFile,
        /// Read buffer using command.
        BufReadCmd,
        /// After reading a buffer.
        BufReadPost,
        /// Before reading a buffer.
        BufReadPre,
        /// Just before unloading a buffer.
        BufUnload,
        /// After showing a buffer in a window.
        BufWinEnter,
        /// Just after buffer removed from window.
        BufWinLeave,
        /// Just before really deleting a buffer.
        BufWipeout,
        /// Write buffer using command.
        BufWriteCmd,
        /// After writing a buffer.
        BufWritePost,
        /// Before writing a buffer.
        BufWritePre,
        /// Info was received about channel.
        ChanInfo,
        /// Channel was opened.
        ChanOpen,
        /// Command undefined.
        CmdUndefined,
        /// After entering the cmdline window.
        CmdWinEnter,
        /// Before leaving the cmdline window.
        CmdWinLeave,
        /// Command line was modified.
        CmdlineChanged,
        /// After entering cmdline mode.
        CmdlineEnter,
        /// Before leaving cmdline mode.
        CmdlineLeave,
        /// After loading a colorscheme.
        ColorScheme,
        /// Before loading a colorscheme.
        ColorSchemePre,
        /// After popup menu changed.
        CompleteChanged,
        /// After finishing insert complete.
        CompleteDone,
        /// Idem, before clearing info.
        CompleteDonePre,
        /// Cursor in same position for a while.
        CursorHold,
        /// Idem, in Insert mode.
        CursorHoldI,
        /// Cursor was moved.
        CursorMoved,
        /// Cursor was moved in Insert mode.
        CursorMovedI,
        /// Diagnostics in a buffer were modified.
        DiagnosticChanged,
        /// Diffs have been updated.
        DiffUpdated,
        /// Directory changed.
        DirChanged,
        /// Directory is going to change.
        DirChangedPre,
        /// After changing the 'encoding' option.
        EncodingChanged,
        /// Before exiting.
        ExitPre,
        /// Append to a file using command.
        FileAppendCmd,
        /// After appending to a file.
        FileAppendPost,
        /// Before appending to a file.
        FileAppendPre,
        /// Before first change to read-only file.
        FileChangedRO,
        /// After shell command that changed file.
        FileChangedShell,
        /// After (not) reloading changed file.
        FileChangedShellPost,
        /// Read from a file using command.
        FileReadCmd,
        /// After reading a file.
        FileReadPost,
        /// Before reading a file.
        FileReadPre,
        /// New file type detected (user defined).
        FileType,
        /// Write to a file using command.
        FileWriteCmd,
        /// After writing a file.
        FileWritePost,
        /// Before writing a file.
        FileWritePre,
        /// After reading from a filter.
        FilterReadPost,
        /// Before reading from a filter.
        FilterReadPre,
        /// After writing to a filter.
        FilterWritePost,
        /// Before writing to a filter.
        FilterWritePre,
        /// Got the focus.
        FocusGained,
        /// Lost the focus to another app.
        FocusLost,
        /// If calling a function which doesn't exist.
        FuncUndefined,
        /// After starting the GUI.
        GUIEnter,
        /// After starting the GUI failed.
        GUIFailed,
        /// When changing Insert/Replace mode.
        InsertChange,
        /// Before inserting a char.
        InsertCharPre,
        /// When entering Insert mode.
        InsertEnter,
        /// Just after leaving Insert mode.
        InsertLeave,
        /// Just before leaving Insert mode.
        InsertLeavePre,
        /// After an LSP client attaches to a buffer.
        LspAttach,
        /// After an LSP client detaches from a buffer.
        LspDetach,
        /// After an LSP request is started, canceled, or completed.
        LspRequest,
        /// After an LSP notice has been sent to the server.
        LspNotify,
        /// After a visible LSP token is updated.
        LspTokenUpdate,
        /// After a LSP progress update.
        LspProgress,
        /// Just before popup menu is displayed.
        MenuPopup,
        /// After changing the mode.
        ModeChanged,
        /// After setting any option.
        OptionSet,
        /// After :make, :grep etc.
        QuickFixCmdPost,
        /// Before :make, :grep etc.
        QuickFixCmdPre,
        /// Before :quit.
        QuitPre,
        /// When starting to record a macro.
        RecordingEnter,
        /// Just before a macro stops recording.
        RecordingLeave,
        /// Upon string reception from a remote vim.
        RemoteReply,
        /// Going to wait for a character.
        SafeState,
        /// After the search wrapped around.
        SearchWrapped,
        /// After loading a session file.
        SessionLoadPost,
        /// After writing a session file.
        SessionWritePost,
        /// After ":!cmd".
        ShellCmdPost,
        /// After ":1,2!cmd", ":w !cmd", ":r !cmd".
        ShellFilterPost,
        /// After nvim process received a signal.
        Signal,
        /// Sourcing a Vim script using command.
        SourceCmd,
        /// After sourcing a Vim script.
        SourcePost,
        /// Before sourcing a Vim script.
        SourcePre,
        /// Spell file missing.
        SpellFileMissing,
        /// After reading from stdin.
        StdinReadPost,
        /// Before reading from stdin.
        StdinReadPre,
        /// Found existing swap file.
        SwapExists,
        /// Syntax selected.
        Syntax,
        /// After a tab has closed.
        TabClosed,
        /// After entering a tab page.
        TabEnter,
        /// Before leaving a tab page.
        TabLeave,
        /// When creating a new tab.
        TabNew,
        /// After entering a new tab.
        TabNewEntered,
        /// After changing 'term'.
        TermChanged,
        /// After the process exits.
        TermClose,
        /// After entering Terminal mode.
        TermEnter,
        /// After leaving Terminal mode.
        TermLeave,
        /// After opening a terminal buffer.
        TermOpen,
        /// After an unhandled OSC sequence is emitted.
        TermRequest,
        /// After setting "v:termresponse".
        TermResponse,
        /// Text was modified.
        TextChanged,
        /// Text was modified in Insert mode(no popup).
        TextChangedI,
        /// Text was modified in Insert mode(popup).
        TextChangedP,
        /// Text was modified in Terminal mode.
        TextChangedT,
        /// After a yank or delete was done (y, d, c).
        TextYankPost,
        /// After UI attaches.
        UIEnter,
        /// After UI detaches.
        UILeave,
        /// User defined autocommand.
        User,
        /// After starting Vim.
        VimEnter,
        /// Before exiting Vim.
        VimLeave,
        /// Before exiting Vim and writing ShaDa file.
        VimLeavePre,
        /// After Vim window was resized.
        VimResized,
        /// After Nvim is resumed.
        VimResume,
        /// Before Nvim is suspended.
        VimSuspend,
        /// After closing a window.
        WinClosed,
        /// After entering a window.
        WinEnter,
        /// Before leaving a window.
        WinLeave,
        /// When entering a new window.
        WinNew,
        /// After a window was resized.
        WinResized,
        /// After a window was scrolled or resized.
        WinScrolled,
    }
    #[automatically_derived]
    impl ::core::fmt::Debug for AutoCmdEvent {
        #[inline]
        fn fmt(&self, f: &mut ::core::fmt::Formatter) -> ::core::fmt::Result {
            ::core::fmt::Formatter::write_str(
                f,
                match self {
                    AutoCmdEvent::BufAdd => "BufAdd",
                    AutoCmdEvent::BufDelete => "BufDelete",
                    AutoCmdEvent::BufEnter => "BufEnter",
                    AutoCmdEvent::BufFilePost => "BufFilePost",
                    AutoCmdEvent::BufFilePre => "BufFilePre",
                    AutoCmdEvent::BufHidden => "BufHidden",
                    AutoCmdEvent::BufLeave => "BufLeave",
                    AutoCmdEvent::BufModifiedSet => "BufModifiedSet",
                    AutoCmdEvent::BufNew => "BufNew",
                    AutoCmdEvent::BufNewFile => "BufNewFile",
                    AutoCmdEvent::BufReadCmd => "BufReadCmd",
                    AutoCmdEvent::BufReadPost => "BufReadPost",
                    AutoCmdEvent::BufReadPre => "BufReadPre",
                    AutoCmdEvent::BufUnload => "BufUnload",
                    AutoCmdEvent::BufWinEnter => "BufWinEnter",
                    AutoCmdEvent::BufWinLeave => "BufWinLeave",
                    AutoCmdEvent::BufWipeout => "BufWipeout",
                    AutoCmdEvent::BufWriteCmd => "BufWriteCmd",
                    AutoCmdEvent::BufWritePost => "BufWritePost",
                    AutoCmdEvent::BufWritePre => "BufWritePre",
                    AutoCmdEvent::ChanInfo => "ChanInfo",
                    AutoCmdEvent::ChanOpen => "ChanOpen",
                    AutoCmdEvent::CmdUndefined => "CmdUndefined",
                    AutoCmdEvent::CmdWinEnter => "CmdWinEnter",
                    AutoCmdEvent::CmdWinLeave => "CmdWinLeave",
                    AutoCmdEvent::CmdlineChanged => "CmdlineChanged",
                    AutoCmdEvent::CmdlineEnter => "CmdlineEnter",
                    AutoCmdEvent::CmdlineLeave => "CmdlineLeave",
                    AutoCmdEvent::ColorScheme => "ColorScheme",
                    AutoCmdEvent::ColorSchemePre => "ColorSchemePre",
                    AutoCmdEvent::CompleteChanged => "CompleteChanged",
                    AutoCmdEvent::CompleteDone => "CompleteDone",
                    AutoCmdEvent::CompleteDonePre => "CompleteDonePre",
                    AutoCmdEvent::CursorHold => "CursorHold",
                    AutoCmdEvent::CursorHoldI => "CursorHoldI",
                    AutoCmdEvent::CursorMoved => "CursorMoved",
                    AutoCmdEvent::CursorMovedI => "CursorMovedI",
                    AutoCmdEvent::DiagnosticChanged => "DiagnosticChanged",
                    AutoCmdEvent::DiffUpdated => "DiffUpdated",
                    AutoCmdEvent::DirChanged => "DirChanged",
                    AutoCmdEvent::DirChangedPre => "DirChangedPre",
                    AutoCmdEvent::EncodingChanged => "EncodingChanged",
                    AutoCmdEvent::ExitPre => "ExitPre",
                    AutoCmdEvent::FileAppendCmd => "FileAppendCmd",
                    AutoCmdEvent::FileAppendPost => "FileAppendPost",
                    AutoCmdEvent::FileAppendPre => "FileAppendPre",
                    AutoCmdEvent::FileChangedRO => "FileChangedRO",
                    AutoCmdEvent::FileChangedShell => "FileChangedShell",
                    AutoCmdEvent::FileChangedShellPost => "FileChangedShellPost",
                    AutoCmdEvent::FileReadCmd => "FileReadCmd",
                    AutoCmdEvent::FileReadPost => "FileReadPost",
                    AutoCmdEvent::FileReadPre => "FileReadPre",
                    AutoCmdEvent::FileType => "FileType",
                    AutoCmdEvent::FileWriteCmd => "FileWriteCmd",
                    AutoCmdEvent::FileWritePost => "FileWritePost",
                    AutoCmdEvent::FileWritePre => "FileWritePre",
                    AutoCmdEvent::FilterReadPost => "FilterReadPost",
                    AutoCmdEvent::FilterReadPre => "FilterReadPre",
                    AutoCmdEvent::FilterWritePost => "FilterWritePost",
                    AutoCmdEvent::FilterWritePre => "FilterWritePre",
                    AutoCmdEvent::FocusGained => "FocusGained",
                    AutoCmdEvent::FocusLost => "FocusLost",
                    AutoCmdEvent::FuncUndefined => "FuncUndefined",
                    AutoCmdEvent::GUIEnter => "GUIEnter",
                    AutoCmdEvent::GUIFailed => "GUIFailed",
                    AutoCmdEvent::InsertChange => "InsertChange",
                    AutoCmdEvent::InsertCharPre => "InsertCharPre",
                    AutoCmdEvent::InsertEnter => "InsertEnter",
                    AutoCmdEvent::InsertLeave => "InsertLeave",
                    AutoCmdEvent::InsertLeavePre => "InsertLeavePre",
                    AutoCmdEvent::LspAttach => "LspAttach",
                    AutoCmdEvent::LspDetach => "LspDetach",
                    AutoCmdEvent::LspRequest => "LspRequest",
                    AutoCmdEvent::LspNotify => "LspNotify",
                    AutoCmdEvent::LspTokenUpdate => "LspTokenUpdate",
                    AutoCmdEvent::LspProgress => "LspProgress",
                    AutoCmdEvent::MenuPopup => "MenuPopup",
                    AutoCmdEvent::ModeChanged => "ModeChanged",
                    AutoCmdEvent::OptionSet => "OptionSet",
                    AutoCmdEvent::QuickFixCmdPost => "QuickFixCmdPost",
                    AutoCmdEvent::QuickFixCmdPre => "QuickFixCmdPre",
                    AutoCmdEvent::QuitPre => "QuitPre",
                    AutoCmdEvent::RecordingEnter => "RecordingEnter",
                    AutoCmdEvent::RecordingLeave => "RecordingLeave",
                    AutoCmdEvent::RemoteReply => "RemoteReply",
                    AutoCmdEvent::SafeState => "SafeState",
                    AutoCmdEvent::SearchWrapped => "SearchWrapped",
                    AutoCmdEvent::SessionLoadPost => "SessionLoadPost",
                    AutoCmdEvent::SessionWritePost => "SessionWritePost",
                    AutoCmdEvent::ShellCmdPost => "ShellCmdPost",
                    AutoCmdEvent::ShellFilterPost => "ShellFilterPost",
                    AutoCmdEvent::Signal => "Signal",
                    AutoCmdEvent::SourceCmd => "SourceCmd",
                    AutoCmdEvent::SourcePost => "SourcePost",
                    AutoCmdEvent::SourcePre => "SourcePre",
                    AutoCmdEvent::SpellFileMissing => "SpellFileMissing",
                    AutoCmdEvent::StdinReadPost => "StdinReadPost",
                    AutoCmdEvent::StdinReadPre => "StdinReadPre",
                    AutoCmdEvent::SwapExists => "SwapExists",
                    AutoCmdEvent::Syntax => "Syntax",
                    AutoCmdEvent::TabClosed => "TabClosed",
                    AutoCmdEvent::TabEnter => "TabEnter",
                    AutoCmdEvent::TabLeave => "TabLeave",
                    AutoCmdEvent::TabNew => "TabNew",
                    AutoCmdEvent::TabNewEntered => "TabNewEntered",
                    AutoCmdEvent::TermChanged => "TermChanged",
                    AutoCmdEvent::TermClose => "TermClose",
                    AutoCmdEvent::TermEnter => "TermEnter",
                    AutoCmdEvent::TermLeave => "TermLeave",
                    AutoCmdEvent::TermOpen => "TermOpen",
                    AutoCmdEvent::TermRequest => "TermRequest",
                    AutoCmdEvent::TermResponse => "TermResponse",
                    AutoCmdEvent::TextChanged => "TextChanged",
                    AutoCmdEvent::TextChangedI => "TextChangedI",
                    AutoCmdEvent::TextChangedP => "TextChangedP",
                    AutoCmdEvent::TextChangedT => "TextChangedT",
                    AutoCmdEvent::TextYankPost => "TextYankPost",
                    AutoCmdEvent::UIEnter => "UIEnter",
                    AutoCmdEvent::UILeave => "UILeave",
                    AutoCmdEvent::User => "User",
                    AutoCmdEvent::VimEnter => "VimEnter",
                    AutoCmdEvent::VimLeave => "VimLeave",
                    AutoCmdEvent::VimLeavePre => "VimLeavePre",
                    AutoCmdEvent::VimResized => "VimResized",
                    AutoCmdEvent::VimResume => "VimResume",
                    AutoCmdEvent::VimSuspend => "VimSuspend",
                    AutoCmdEvent::WinClosed => "WinClosed",
                    AutoCmdEvent::WinEnter => "WinEnter",
                    AutoCmdEvent::WinLeave => "WinLeave",
                    AutoCmdEvent::WinNew => "WinNew",
                    AutoCmdEvent::WinResized => "WinResized",
                    AutoCmdEvent::WinScrolled => "WinScrolled",
                },
            )
        }
    }
    #[automatically_derived]
    impl ::core::clone::Clone for AutoCmdEvent {
        #[inline]
        fn clone(&self) -> AutoCmdEvent {
            *self
        }
    }
    #[automatically_derived]
    impl ::core::marker::Copy for AutoCmdEvent {}
    impl std::fmt::Display for AutoCmdEvent {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            match self {
                Self::BufAdd => f.write_str("BufAdd"),
                Self::BufDelete => f.write_str("BufDelete"),
                Self::BufEnter => f.write_str("BufEnter"),
                Self::BufFilePost => f.write_str("BufFilePost"),
                Self::BufFilePre => f.write_str("BufFilePre"),
                Self::BufHidden => f.write_str("BufHidden"),
                Self::BufLeave => f.write_str("BufLeave"),
                Self::BufModifiedSet => f.write_str("BufModifiedSet"),
                Self::BufNew => f.write_str("BufNew"),
                Self::BufNewFile => f.write_str("BufNewFile"),
                Self::BufReadCmd => f.write_str("BufReadCmd"),
                Self::BufReadPost => f.write_str("BufReadPost"),
                Self::BufReadPre => f.write_str("BufReadPre"),
                Self::BufUnload => f.write_str("BufUnload"),
                Self::BufWinEnter => f.write_str("BufWinEnter"),
                Self::BufWinLeave => f.write_str("BufWinLeave"),
                Self::BufWipeout => f.write_str("BufWipeout"),
                Self::BufWriteCmd => f.write_str("BufWriteCmd"),
                Self::BufWritePost => f.write_str("BufWritePost"),
                Self::BufWritePre => f.write_str("BufWritePre"),
                Self::ChanInfo => f.write_str("ChanInfo"),
                Self::ChanOpen => f.write_str("ChanOpen"),
                Self::CmdUndefined => f.write_str("CmdUndefined"),
                Self::CmdWinEnter => f.write_str("CmdWinEnter"),
                Self::CmdWinLeave => f.write_str("CmdWinLeave"),
                Self::CmdlineChanged => f.write_str("CmdlineChanged"),
                Self::CmdlineEnter => f.write_str("CmdlineEnter"),
                Self::CmdlineLeave => f.write_str("CmdlineLeave"),
                Self::ColorScheme => f.write_str("ColorScheme"),
                Self::ColorSchemePre => f.write_str("ColorSchemePre"),
                Self::CompleteChanged => f.write_str("CompleteChanged"),
                Self::CompleteDone => f.write_str("CompleteDone"),
                Self::CompleteDonePre => f.write_str("CompleteDonePre"),
                Self::CursorHold => f.write_str("CursorHold"),
                Self::CursorHoldI => f.write_str("CursorHoldI"),
                Self::CursorMoved => f.write_str("CursorMoved"),
                Self::CursorMovedI => f.write_str("CursorMovedI"),
                Self::DiagnosticChanged => f.write_str("DiagnosticChanged"),
                Self::DiffUpdated => f.write_str("DiffUpdated"),
                Self::DirChanged => f.write_str("DirChanged"),
                Self::DirChangedPre => f.write_str("DirChangedPre"),
                Self::EncodingChanged => f.write_str("EncodingChanged"),
                Self::ExitPre => f.write_str("ExitPre"),
                Self::FileAppendCmd => f.write_str("FileAppendCmd"),
                Self::FileAppendPost => f.write_str("FileAppendPost"),
                Self::FileAppendPre => f.write_str("FileAppendPre"),
                Self::FileChangedRO => f.write_str("FileChangedRO"),
                Self::FileChangedShell => f.write_str("FileChangedShell"),
                Self::FileChangedShellPost => f.write_str("FileChangedShellPost"),
                Self::FileReadCmd => f.write_str("FileReadCmd"),
                Self::FileReadPost => f.write_str("FileReadPost"),
                Self::FileReadPre => f.write_str("FileReadPre"),
                Self::FileType => f.write_str("FileType"),
                Self::FileWriteCmd => f.write_str("FileWriteCmd"),
                Self::FileWritePost => f.write_str("FileWritePost"),
                Self::FileWritePre => f.write_str("FileWritePre"),
                Self::FilterReadPost => f.write_str("FilterReadPost"),
                Self::FilterReadPre => f.write_str("FilterReadPre"),
                Self::FilterWritePost => f.write_str("FilterWritePost"),
                Self::FilterWritePre => f.write_str("FilterWritePre"),
                Self::FocusGained => f.write_str("FocusGained"),
                Self::FocusLost => f.write_str("FocusLost"),
                Self::FuncUndefined => f.write_str("FuncUndefined"),
                Self::GUIEnter => f.write_str("GUIEnter"),
                Self::GUIFailed => f.write_str("GUIFailed"),
                Self::InsertChange => f.write_str("InsertChange"),
                Self::InsertCharPre => f.write_str("InsertCharPre"),
                Self::InsertEnter => f.write_str("InsertEnter"),
                Self::InsertLeave => f.write_str("InsertLeave"),
                Self::InsertLeavePre => f.write_str("InsertLeavePre"),
                Self::LspAttach => f.write_str("LspAttach"),
                Self::LspDetach => f.write_str("LspDetach"),
                Self::LspRequest => f.write_str("LspRequest"),
                Self::LspNotify => f.write_str("LspNotify"),
                Self::LspTokenUpdate => f.write_str("LspTokenUpdate"),
                Self::LspProgress => f.write_str("LspProgress"),
                Self::MenuPopup => f.write_str("MenuPopup"),
                Self::ModeChanged => f.write_str("ModeChanged"),
                Self::OptionSet => f.write_str("OptionSet"),
                Self::QuickFixCmdPost => f.write_str("QuickFixCmdPost"),
                Self::QuickFixCmdPre => f.write_str("QuickFixCmdPre"),
                Self::QuitPre => f.write_str("QuitPre"),
                Self::RecordingEnter => f.write_str("RecordingEnter"),
                Self::RecordingLeave => f.write_str("RecordingLeave"),
                Self::RemoteReply => f.write_str("RemoteReply"),
                Self::SafeState => f.write_str("SafeState"),
                Self::SearchWrapped => f.write_str("SearchWrapped"),
                Self::SessionLoadPost => f.write_str("SessionLoadPost"),
                Self::SessionWritePost => f.write_str("SessionWritePost"),
                Self::ShellCmdPost => f.write_str("ShellCmdPost"),
                Self::ShellFilterPost => f.write_str("ShellFilterPost"),
                Self::Signal => f.write_str("Signal"),
                Self::SourceCmd => f.write_str("SourceCmd"),
                Self::SourcePost => f.write_str("SourcePost"),
                Self::SourcePre => f.write_str("SourcePre"),
                Self::SpellFileMissing => f.write_str("SpellFileMissing"),
                Self::StdinReadPost => f.write_str("StdinReadPost"),
                Self::StdinReadPre => f.write_str("StdinReadPre"),
                Self::SwapExists => f.write_str("SwapExists"),
                Self::Syntax => f.write_str("Syntax"),
                Self::TabClosed => f.write_str("TabClosed"),
                Self::TabEnter => f.write_str("TabEnter"),
                Self::TabLeave => f.write_str("TabLeave"),
                Self::TabNew => f.write_str("TabNew"),
                Self::TabNewEntered => f.write_str("TabNewEntered"),
                Self::TermChanged => f.write_str("TermChanged"),
                Self::TermClose => f.write_str("TermClose"),
                Self::TermEnter => f.write_str("TermEnter"),
                Self::TermLeave => f.write_str("TermLeave"),
                Self::TermOpen => f.write_str("TermOpen"),
                Self::TermRequest => f.write_str("TermRequest"),
                Self::TermResponse => f.write_str("TermResponse"),
                Self::TextChanged => f.write_str("TextChanged"),
                Self::TextChangedI => f.write_str("TextChangedI"),
                Self::TextChangedP => f.write_str("TextChangedP"),
                Self::TextChangedT => f.write_str("TextChangedT"),
                Self::TextYankPost => f.write_str("TextYankPost"),
                Self::UIEnter => f.write_str("UIEnter"),
                Self::UILeave => f.write_str("UILeave"),
                Self::User => f.write_str("User"),
                Self::VimEnter => f.write_str("VimEnter"),
                Self::VimLeave => f.write_str("VimLeave"),
                Self::VimLeavePre => f.write_str("VimLeavePre"),
                Self::VimResized => f.write_str("VimResized"),
                Self::VimResume => f.write_str("VimResume"),
                Self::VimSuspend => f.write_str("VimSuspend"),
                Self::WinClosed => f.write_str("WinClosed"),
                Self::WinEnter => f.write_str("WinEnter"),
                Self::WinLeave => f.write_str("WinLeave"),
                Self::WinNew => f.write_str("WinNew"),
                Self::WinResized => f.write_str("WinResized"),
                Self::WinScrolled => f.write_str("WinScrolled"),
            }
        }
    }
    impl<'a> mlua::IntoLua<'a> for AutoCmdEvent {
        fn into_lua(self, lua: &'a Lua) -> mlua::Result<mlua::Value<'a>> {
            let str = lua.create_string(self.to_string())?;
            Ok(mlua::Value::String(str))
        }
    }
    #[automatically_derived]
    impl ::core::marker::StructuralPartialEq for AutoCmdEvent {}
    #[automatically_derived]
    impl ::core::cmp::PartialEq for AutoCmdEvent {
        #[inline]
        fn eq(&self, other: &AutoCmdEvent) -> bool {
            let __self_tag = ::core::intrinsics::discriminant_value(self);
            let __arg1_tag = ::core::intrinsics::discriminant_value(other);
            __self_tag == __arg1_tag
        }
    }
    #[automatically_derived]
    impl ::core::cmp::Eq for AutoCmdEvent {
        #[inline]
        #[doc(hidden)]
        #[coverage(off)]
        fn assert_receiver_is_total_eq(&self) -> () {}
    }
    pub struct AutoCmdOpts<'a> {
        /// Autocommand group name or id to match against.
        pub group: Option<AutoCmdGroup>,
        /// Optional: pattern(s) to match literally |autocmd-pattern|.
        pub pattern: Vec<String>,
        /// Optional: buffer number for buffer-local
        /// autocommands |autocmd-buflocal|. Cannot be used with {pattern}
        pub buffer: Option<u32>,
        /// description (for documentation and troubleshooting).
        pub desc: Option<String>,
        /**
    Lua function called when the event(s) is triggered.
    Lua callback can return a truthy value (not `false` or `nil`) to delete the autocommand.
    Receives a table argument with these keys:
    • id: (number) autocommand id
    • event: (string) name of the triggered event |autocmd-events|
    • group: (number|nil) autocommand group id, if any
    • match: (string) expanded value of <amatch>
    • buf: (number) expanded value of <abuf>
    • file: (string) expanded value of <afile>
    • data: (any) arbitrary data passed from |nvim_exec_autocmds()|
     */
        pub callback: LuaFunction<'a>,
        /// defaults to false. Run the autocommand only once |autocmd-once|.
        pub once: bool,
    }
    #[automatically_derived]
    impl<'a> ::core::fmt::Debug for AutoCmdOpts<'a> {
        #[inline]
        fn fmt(&self, f: &mut ::core::fmt::Formatter) -> ::core::fmt::Result {
            let names: &'static _ = &[
                "group",
                "pattern",
                "buffer",
                "desc",
                "callback",
                "once",
            ];
            let values: &[&dyn ::core::fmt::Debug] = &[
                &self.group,
                &self.pattern,
                &self.buffer,
                &self.desc,
                &self.callback,
                &&self.once,
            ];
            ::core::fmt::Formatter::debug_struct_fields_finish(
                f,
                "AutoCmdOpts",
                names,
                values,
            )
        }
    }
    pub struct AutoCmdCbEvent {
        /// Autocommand id
        pub id: u32,
        /// Name of the triggered event |autocmd-events|
        pub event: String,
        /// Autocommand group id, if any
        pub group: Option<u32>,
        /// Expanded value of <amatch>
        pub r#match: String,
        /// Expanded value of <abuf>
        pub buf: Option<u32>,
        /// Expanded value of <afile>
        pub file: String,
        ///  (Any) arbitrary data passed from |nvim_exec_autocmds()|
        /// You can use CbDataFiller type in the callback function if you don't need any data
        pub data: Option<usize>,
    }
    #[automatically_derived]
    impl ::core::fmt::Debug for AutoCmdCbEvent {
        #[inline]
        fn fmt(&self, f: &mut ::core::fmt::Formatter) -> ::core::fmt::Result {
            let names: &'static _ = &[
                "id",
                "event",
                "group",
                "match",
                "buf",
                "file",
                "data",
            ];
            let values: &[&dyn ::core::fmt::Debug] = &[
                &self.id,
                &self.event,
                &self.group,
                &self.r#match,
                &self.buf,
                &self.file,
                &&self.data,
            ];
            ::core::fmt::Formatter::debug_struct_fields_finish(
                f,
                "AutoCmdCbEvent",
                names,
                values,
            )
        }
    }
    #[automatically_derived]
    impl ::core::clone::Clone for AutoCmdCbEvent {
        #[inline]
        fn clone(&self) -> AutoCmdCbEvent {
            AutoCmdCbEvent {
                id: ::core::clone::Clone::clone(&self.id),
                event: ::core::clone::Clone::clone(&self.event),
                group: ::core::clone::Clone::clone(&self.group),
                r#match: ::core::clone::Clone::clone(&self.r#match),
                buf: ::core::clone::Clone::clone(&self.buf),
                file: ::core::clone::Clone::clone(&self.file),
                data: ::core::clone::Clone::clone(&self.data),
            }
        }
    }
    #[doc(hidden)]
    #[allow(non_upper_case_globals, unused_attributes, unused_qualifications)]
    const _: () = {
        #[allow(unused_extern_crates, clippy::useless_attribute)]
        extern crate serde as _serde;
        #[automatically_derived]
        impl<'de> _serde::Deserialize<'de> for AutoCmdCbEvent {
            fn deserialize<__D>(
                __deserializer: __D,
            ) -> _serde::__private::Result<Self, __D::Error>
            where
                __D: _serde::Deserializer<'de>,
            {
                #[allow(non_camel_case_types)]
                #[doc(hidden)]
                enum __Field {
                    __field0,
                    __field1,
                    __field2,
                    __field3,
                    __field4,
                    __field5,
                    __field6,
                    __ignore,
                }
                #[doc(hidden)]
                struct __FieldVisitor;
                impl<'de> _serde::de::Visitor<'de> for __FieldVisitor {
                    type Value = __Field;
                    fn expecting(
                        &self,
                        __formatter: &mut _serde::__private::Formatter,
                    ) -> _serde::__private::fmt::Result {
                        _serde::__private::Formatter::write_str(
                            __formatter,
                            "field identifier",
                        )
                    }
                    fn visit_u64<__E>(
                        self,
                        __value: u64,
                    ) -> _serde::__private::Result<Self::Value, __E>
                    where
                        __E: _serde::de::Error,
                    {
                        match __value {
                            0u64 => _serde::__private::Ok(__Field::__field0),
                            1u64 => _serde::__private::Ok(__Field::__field1),
                            2u64 => _serde::__private::Ok(__Field::__field2),
                            3u64 => _serde::__private::Ok(__Field::__field3),
                            4u64 => _serde::__private::Ok(__Field::__field4),
                            5u64 => _serde::__private::Ok(__Field::__field5),
                            6u64 => _serde::__private::Ok(__Field::__field6),
                            _ => _serde::__private::Ok(__Field::__ignore),
                        }
                    }
                    fn visit_str<__E>(
                        self,
                        __value: &str,
                    ) -> _serde::__private::Result<Self::Value, __E>
                    where
                        __E: _serde::de::Error,
                    {
                        match __value {
                            "id" => _serde::__private::Ok(__Field::__field0),
                            "event" => _serde::__private::Ok(__Field::__field1),
                            "group" => _serde::__private::Ok(__Field::__field2),
                            "match" => _serde::__private::Ok(__Field::__field3),
                            "buf" => _serde::__private::Ok(__Field::__field4),
                            "file" => _serde::__private::Ok(__Field::__field5),
                            "data" => _serde::__private::Ok(__Field::__field6),
                            _ => _serde::__private::Ok(__Field::__ignore),
                        }
                    }
                    fn visit_bytes<__E>(
                        self,
                        __value: &[u8],
                    ) -> _serde::__private::Result<Self::Value, __E>
                    where
                        __E: _serde::de::Error,
                    {
                        match __value {
                            b"id" => _serde::__private::Ok(__Field::__field0),
                            b"event" => _serde::__private::Ok(__Field::__field1),
                            b"group" => _serde::__private::Ok(__Field::__field2),
                            b"match" => _serde::__private::Ok(__Field::__field3),
                            b"buf" => _serde::__private::Ok(__Field::__field4),
                            b"file" => _serde::__private::Ok(__Field::__field5),
                            b"data" => _serde::__private::Ok(__Field::__field6),
                            _ => _serde::__private::Ok(__Field::__ignore),
                        }
                    }
                }
                impl<'de> _serde::Deserialize<'de> for __Field {
                    #[inline]
                    fn deserialize<__D>(
                        __deserializer: __D,
                    ) -> _serde::__private::Result<Self, __D::Error>
                    where
                        __D: _serde::Deserializer<'de>,
                    {
                        _serde::Deserializer::deserialize_identifier(
                            __deserializer,
                            __FieldVisitor,
                        )
                    }
                }
                #[doc(hidden)]
                struct __Visitor<'de> {
                    marker: _serde::__private::PhantomData<AutoCmdCbEvent>,
                    lifetime: _serde::__private::PhantomData<&'de ()>,
                }
                impl<'de> _serde::de::Visitor<'de> for __Visitor<'de> {
                    type Value = AutoCmdCbEvent;
                    fn expecting(
                        &self,
                        __formatter: &mut _serde::__private::Formatter,
                    ) -> _serde::__private::fmt::Result {
                        _serde::__private::Formatter::write_str(
                            __formatter,
                            "struct AutoCmdCbEvent",
                        )
                    }
                    #[inline]
                    fn visit_seq<__A>(
                        self,
                        mut __seq: __A,
                    ) -> _serde::__private::Result<Self::Value, __A::Error>
                    where
                        __A: _serde::de::SeqAccess<'de>,
                    {
                        let __field0 = match _serde::de::SeqAccess::next_element::<
                            u32,
                        >(&mut __seq)? {
                            _serde::__private::Some(__value) => __value,
                            _serde::__private::None => {
                                return _serde::__private::Err(
                                    _serde::de::Error::invalid_length(
                                        0usize,
                                        &"struct AutoCmdCbEvent with 7 elements",
                                    ),
                                );
                            }
                        };
                        let __field1 = match _serde::de::SeqAccess::next_element::<
                            String,
                        >(&mut __seq)? {
                            _serde::__private::Some(__value) => __value,
                            _serde::__private::None => {
                                return _serde::__private::Err(
                                    _serde::de::Error::invalid_length(
                                        1usize,
                                        &"struct AutoCmdCbEvent with 7 elements",
                                    ),
                                );
                            }
                        };
                        let __field2 = match _serde::de::SeqAccess::next_element::<
                            Option<u32>,
                        >(&mut __seq)? {
                            _serde::__private::Some(__value) => __value,
                            _serde::__private::None => {
                                return _serde::__private::Err(
                                    _serde::de::Error::invalid_length(
                                        2usize,
                                        &"struct AutoCmdCbEvent with 7 elements",
                                    ),
                                );
                            }
                        };
                        let __field3 = match _serde::de::SeqAccess::next_element::<
                            String,
                        >(&mut __seq)? {
                            _serde::__private::Some(__value) => __value,
                            _serde::__private::None => {
                                return _serde::__private::Err(
                                    _serde::de::Error::invalid_length(
                                        3usize,
                                        &"struct AutoCmdCbEvent with 7 elements",
                                    ),
                                );
                            }
                        };
                        let __field4 = match _serde::de::SeqAccess::next_element::<
                            Option<u32>,
                        >(&mut __seq)? {
                            _serde::__private::Some(__value) => __value,
                            _serde::__private::None => {
                                return _serde::__private::Err(
                                    _serde::de::Error::invalid_length(
                                        4usize,
                                        &"struct AutoCmdCbEvent with 7 elements",
                                    ),
                                );
                            }
                        };
                        let __field5 = match _serde::de::SeqAccess::next_element::<
                            String,
                        >(&mut __seq)? {
                            _serde::__private::Some(__value) => __value,
                            _serde::__private::None => {
                                return _serde::__private::Err(
                                    _serde::de::Error::invalid_length(
                                        5usize,
                                        &"struct AutoCmdCbEvent with 7 elements",
                                    ),
                                );
                            }
                        };
                        let __field6 = match _serde::de::SeqAccess::next_element::<
                            Option<usize>,
                        >(&mut __seq)? {
                            _serde::__private::Some(__value) => __value,
                            _serde::__private::None => {
                                return _serde::__private::Err(
                                    _serde::de::Error::invalid_length(
                                        6usize,
                                        &"struct AutoCmdCbEvent with 7 elements",
                                    ),
                                );
                            }
                        };
                        _serde::__private::Ok(AutoCmdCbEvent {
                            id: __field0,
                            event: __field1,
                            group: __field2,
                            r#match: __field3,
                            buf: __field4,
                            file: __field5,
                            data: __field6,
                        })
                    }
                    #[inline]
                    fn visit_map<__A>(
                        self,
                        mut __map: __A,
                    ) -> _serde::__private::Result<Self::Value, __A::Error>
                    where
                        __A: _serde::de::MapAccess<'de>,
                    {
                        let mut __field0: _serde::__private::Option<u32> = _serde::__private::None;
                        let mut __field1: _serde::__private::Option<String> = _serde::__private::None;
                        let mut __field2: _serde::__private::Option<Option<u32>> = _serde::__private::None;
                        let mut __field3: _serde::__private::Option<String> = _serde::__private::None;
                        let mut __field4: _serde::__private::Option<Option<u32>> = _serde::__private::None;
                        let mut __field5: _serde::__private::Option<String> = _serde::__private::None;
                        let mut __field6: _serde::__private::Option<Option<usize>> = _serde::__private::None;
                        while let _serde::__private::Some(__key) = _serde::de::MapAccess::next_key::<
                            __Field,
                        >(&mut __map)? {
                            match __key {
                                __Field::__field0 => {
                                    if _serde::__private::Option::is_some(&__field0) {
                                        return _serde::__private::Err(
                                            <__A::Error as _serde::de::Error>::duplicate_field("id"),
                                        );
                                    }
                                    __field0 = _serde::__private::Some(
                                        _serde::de::MapAccess::next_value::<u32>(&mut __map)?,
                                    );
                                }
                                __Field::__field1 => {
                                    if _serde::__private::Option::is_some(&__field1) {
                                        return _serde::__private::Err(
                                            <__A::Error as _serde::de::Error>::duplicate_field("event"),
                                        );
                                    }
                                    __field1 = _serde::__private::Some(
                                        _serde::de::MapAccess::next_value::<String>(&mut __map)?,
                                    );
                                }
                                __Field::__field2 => {
                                    if _serde::__private::Option::is_some(&__field2) {
                                        return _serde::__private::Err(
                                            <__A::Error as _serde::de::Error>::duplicate_field("group"),
                                        );
                                    }
                                    __field2 = _serde::__private::Some(
                                        _serde::de::MapAccess::next_value::<
                                            Option<u32>,
                                        >(&mut __map)?,
                                    );
                                }
                                __Field::__field3 => {
                                    if _serde::__private::Option::is_some(&__field3) {
                                        return _serde::__private::Err(
                                            <__A::Error as _serde::de::Error>::duplicate_field("match"),
                                        );
                                    }
                                    __field3 = _serde::__private::Some(
                                        _serde::de::MapAccess::next_value::<String>(&mut __map)?,
                                    );
                                }
                                __Field::__field4 => {
                                    if _serde::__private::Option::is_some(&__field4) {
                                        return _serde::__private::Err(
                                            <__A::Error as _serde::de::Error>::duplicate_field("buf"),
                                        );
                                    }
                                    __field4 = _serde::__private::Some(
                                        _serde::de::MapAccess::next_value::<
                                            Option<u32>,
                                        >(&mut __map)?,
                                    );
                                }
                                __Field::__field5 => {
                                    if _serde::__private::Option::is_some(&__field5) {
                                        return _serde::__private::Err(
                                            <__A::Error as _serde::de::Error>::duplicate_field("file"),
                                        );
                                    }
                                    __field5 = _serde::__private::Some(
                                        _serde::de::MapAccess::next_value::<String>(&mut __map)?,
                                    );
                                }
                                __Field::__field6 => {
                                    if _serde::__private::Option::is_some(&__field6) {
                                        return _serde::__private::Err(
                                            <__A::Error as _serde::de::Error>::duplicate_field("data"),
                                        );
                                    }
                                    __field6 = _serde::__private::Some(
                                        _serde::de::MapAccess::next_value::<
                                            Option<usize>,
                                        >(&mut __map)?,
                                    );
                                }
                                _ => {
                                    let _ = _serde::de::MapAccess::next_value::<
                                        _serde::de::IgnoredAny,
                                    >(&mut __map)?;
                                }
                            }
                        }
                        let __field0 = match __field0 {
                            _serde::__private::Some(__field0) => __field0,
                            _serde::__private::None => {
                                _serde::__private::de::missing_field("id")?
                            }
                        };
                        let __field1 = match __field1 {
                            _serde::__private::Some(__field1) => __field1,
                            _serde::__private::None => {
                                _serde::__private::de::missing_field("event")?
                            }
                        };
                        let __field2 = match __field2 {
                            _serde::__private::Some(__field2) => __field2,
                            _serde::__private::None => {
                                _serde::__private::de::missing_field("group")?
                            }
                        };
                        let __field3 = match __field3 {
                            _serde::__private::Some(__field3) => __field3,
                            _serde::__private::None => {
                                _serde::__private::de::missing_field("match")?
                            }
                        };
                        let __field4 = match __field4 {
                            _serde::__private::Some(__field4) => __field4,
                            _serde::__private::None => {
                                _serde::__private::de::missing_field("buf")?
                            }
                        };
                        let __field5 = match __field5 {
                            _serde::__private::Some(__field5) => __field5,
                            _serde::__private::None => {
                                _serde::__private::de::missing_field("file")?
                            }
                        };
                        let __field6 = match __field6 {
                            _serde::__private::Some(__field6) => __field6,
                            _serde::__private::None => {
                                _serde::__private::de::missing_field("data")?
                            }
                        };
                        _serde::__private::Ok(AutoCmdCbEvent {
                            id: __field0,
                            event: __field1,
                            group: __field2,
                            r#match: __field3,
                            buf: __field4,
                            file: __field5,
                            data: __field6,
                        })
                    }
                }
                #[doc(hidden)]
                const FIELDS: &'static [&'static str] = &[
                    "id",
                    "event",
                    "group",
                    "match",
                    "buf",
                    "file",
                    "data",
                ];
                _serde::Deserializer::deserialize_struct(
                    __deserializer,
                    "AutoCmdCbEvent",
                    FIELDS,
                    __Visitor {
                        marker: _serde::__private::PhantomData::<AutoCmdCbEvent>,
                        lifetime: _serde::__private::PhantomData,
                    },
                )
            }
        }
    };
    impl<'a> FromLua<'a> for AutoCmdCbEvent {
        fn from_lua(value: LuaValue<'a>, lua: &'a Lua) -> LuaResult<Self> {
            lua.from_value(value)
        }
    }
    impl<'a> IntoLua<'a> for AutoCmdOpts<'a> {
        fn into_lua(self, lua: &'a Lua) -> LuaResult<LuaValue<'a>> {
            let table = lua.create_table()?;
            match self.group {
                Some(AutoCmdGroup::String(name)) => table.set("group", name)?,
                Some(AutoCmdGroup::Integer(id)) => table.set("group", id)?,
                None => {}
            };
            if !self.pattern.is_empty() {
                table.set("pattern", self.pattern)?;
            }
            if let Some(buf_id) = self.buffer {
                table.set("buffer", buf_id)?;
            }
            if let Some(desc) = self.desc {
                table.set("desc", desc)?;
            }
            table.set("callback", self.callback)?;
            table.set("once", self.once)?;
            Ok(LuaValue::Table(table))
        }
    }
    pub struct CmdOptsMagic {
        pub bar: bool,
        pub file: bool,
    }
    #[automatically_derived]
    impl ::core::default::Default for CmdOptsMagic {
        #[inline]
        fn default() -> CmdOptsMagic {
            CmdOptsMagic {
                bar: ::core::default::Default::default(),
                file: ::core::default::Default::default(),
            }
        }
    }
    #[automatically_derived]
    impl ::core::clone::Clone for CmdOptsMagic {
        #[inline]
        fn clone(&self) -> CmdOptsMagic {
            CmdOptsMagic {
                bar: ::core::clone::Clone::clone(&self.bar),
                file: ::core::clone::Clone::clone(&self.file),
            }
        }
    }
    pub struct CmdOptsModsFilter<'a> {
        pub force: bool,
        pub pattern: &'a str,
    }
    #[automatically_derived]
    impl<'a> ::core::default::Default for CmdOptsModsFilter<'a> {
        #[inline]
        fn default() -> CmdOptsModsFilter<'a> {
            CmdOptsModsFilter {
                force: ::core::default::Default::default(),
                pattern: ::core::default::Default::default(),
            }
        }
    }
    #[automatically_derived]
    impl<'a> ::core::clone::Clone for CmdOptsModsFilter<'a> {
        #[inline]
        fn clone(&self) -> CmdOptsModsFilter<'a> {
            CmdOptsModsFilter {
                force: ::core::clone::Clone::clone(&self.force),
                pattern: ::core::clone::Clone::clone(&self.pattern),
            }
        }
    }
    pub struct CmdOptsMods<'a> {
        pub browse: bool,
        pub confirm: bool,
        pub emsg_silent: bool,
        pub filter: CmdOptsModsFilter<'a>,
        pub hide: bool,
        pub horizontal: bool,
        pub keepalt: bool,
        pub keepjumps: bool,
        pub keepmarks: bool,
        pub keeppatterns: bool,
        pub lockmarks: bool,
        pub noautocmd: bool,
        pub noswapfile: bool,
        pub sandbox: bool,
        pub silent: bool,
        pub split: &'a str,
        pub tab: i32,
        pub unsilent: bool,
        pub verbose: i32,
        pub vertical: bool,
    }
    #[automatically_derived]
    impl<'a> ::core::clone::Clone for CmdOptsMods<'a> {
        #[inline]
        fn clone(&self) -> CmdOptsMods<'a> {
            CmdOptsMods {
                browse: ::core::clone::Clone::clone(&self.browse),
                confirm: ::core::clone::Clone::clone(&self.confirm),
                emsg_silent: ::core::clone::Clone::clone(&self.emsg_silent),
                filter: ::core::clone::Clone::clone(&self.filter),
                hide: ::core::clone::Clone::clone(&self.hide),
                horizontal: ::core::clone::Clone::clone(&self.horizontal),
                keepalt: ::core::clone::Clone::clone(&self.keepalt),
                keepjumps: ::core::clone::Clone::clone(&self.keepjumps),
                keepmarks: ::core::clone::Clone::clone(&self.keepmarks),
                keeppatterns: ::core::clone::Clone::clone(&self.keeppatterns),
                lockmarks: ::core::clone::Clone::clone(&self.lockmarks),
                noautocmd: ::core::clone::Clone::clone(&self.noautocmd),
                noswapfile: ::core::clone::Clone::clone(&self.noswapfile),
                sandbox: ::core::clone::Clone::clone(&self.sandbox),
                silent: ::core::clone::Clone::clone(&self.silent),
                split: ::core::clone::Clone::clone(&self.split),
                tab: ::core::clone::Clone::clone(&self.tab),
                unsilent: ::core::clone::Clone::clone(&self.unsilent),
                verbose: ::core::clone::Clone::clone(&self.verbose),
                vertical: ::core::clone::Clone::clone(&self.vertical),
            }
        }
    }
    impl Default for CmdOptsMods<'_> {
        fn default() -> Self {
            Self {
                browse: false,
                confirm: false,
                emsg_silent: false,
                filter: CmdOptsModsFilter::default(),
                hide: false,
                horizontal: false,
                keepalt: false,
                keepjumps: false,
                keepmarks: false,
                keeppatterns: false,
                lockmarks: false,
                noautocmd: false,
                noswapfile: false,
                sandbox: false,
                silent: false,
                split: "",
                tab: -1,
                unsilent: false,
                verbose: -1,
                vertical: false,
            }
        }
    }
    impl<'lua, 'opts> IntoLua<'lua> for CmdOpts<'opts> {
        fn into_lua(self, lua: &'lua Lua) -> LuaResult<LuaValue<'lua>> {
            let out = lua.create_table()?;
            out.set("cmd", self.cmd)?;
            out.set("args", self.args)?;
            out.set("bang", self.bang)?;
            Ok(LuaValue::Table(out))
        }
    }
    pub struct CmdOpts<'a> {
        pub cmd: &'a str,
        pub args: &'a [&'a str],
        pub bang: bool,
    }
    #[automatically_derived]
    impl<'a> ::core::clone::Clone for CmdOpts<'a> {
        #[inline]
        fn clone(&self) -> CmdOpts<'a> {
            CmdOpts {
                cmd: ::core::clone::Clone::clone(&self.cmd),
                args: ::core::clone::Clone::clone(&self.args),
                bang: ::core::clone::Clone::clone(&self.bang),
            }
        }
    }
}
mod popup {
    use crate::{HLText, NeoApi, NeoBuffer, NeoWindow, TextType};
    use mlua::{
        prelude::{LuaFunction, LuaResult, LuaValue},
        FromLua, IntoLua, Lua,
    };
    use serde::Serialize;
    use std::{
        fmt::{self, Display},
        time::Duration,
    };
    pub enum PopupRelative {
        #[default]
        Win,
        Cursor,
        Editor,
        Mouse,
    }
    #[automatically_derived]
    impl ::core::fmt::Debug for PopupRelative {
        #[inline]
        fn fmt(&self, f: &mut ::core::fmt::Formatter) -> ::core::fmt::Result {
            ::core::fmt::Formatter::write_str(
                f,
                match self {
                    PopupRelative::Win => "Win",
                    PopupRelative::Cursor => "Cursor",
                    PopupRelative::Editor => "Editor",
                    PopupRelative::Mouse => "Mouse",
                },
            )
        }
    }
    #[automatically_derived]
    impl ::core::default::Default for PopupRelative {
        #[inline]
        fn default() -> PopupRelative {
            Self::Win
        }
    }
    #[automatically_derived]
    impl ::core::clone::Clone for PopupRelative {
        #[inline]
        fn clone(&self) -> PopupRelative {
            *self
        }
    }
    #[automatically_derived]
    impl ::core::marker::Copy for PopupRelative {}
    #[automatically_derived]
    impl ::core::marker::StructuralPartialEq for PopupRelative {}
    #[automatically_derived]
    impl ::core::cmp::PartialEq for PopupRelative {
        #[inline]
        fn eq(&self, other: &PopupRelative) -> bool {
            let __self_tag = ::core::intrinsics::discriminant_value(self);
            let __arg1_tag = ::core::intrinsics::discriminant_value(other);
            __self_tag == __arg1_tag
        }
    }
    #[automatically_derived]
    impl ::core::cmp::Eq for PopupRelative {
        #[inline]
        #[doc(hidden)]
        #[coverage(off)]
        fn assert_receiver_is_total_eq(&self) -> () {}
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
    pub enum Anchor {
        #[default]
        NorthWest,
        NorthEast,
        SouthWest,
        SouthEast,
    }
    #[automatically_derived]
    impl ::core::fmt::Debug for Anchor {
        #[inline]
        fn fmt(&self, f: &mut ::core::fmt::Formatter) -> ::core::fmt::Result {
            ::core::fmt::Formatter::write_str(
                f,
                match self {
                    Anchor::NorthWest => "NorthWest",
                    Anchor::NorthEast => "NorthEast",
                    Anchor::SouthWest => "SouthWest",
                    Anchor::SouthEast => "SouthEast",
                },
            )
        }
    }
    #[automatically_derived]
    impl ::core::default::Default for Anchor {
        #[inline]
        fn default() -> Anchor {
            Self::NorthWest
        }
    }
    #[automatically_derived]
    impl ::core::clone::Clone for Anchor {
        #[inline]
        fn clone(&self) -> Anchor {
            *self
        }
    }
    #[automatically_derived]
    impl ::core::marker::Copy for Anchor {}
    #[automatically_derived]
    impl ::core::marker::StructuralPartialEq for Anchor {}
    #[automatically_derived]
    impl ::core::cmp::PartialEq for Anchor {
        #[inline]
        fn eq(&self, other: &Anchor) -> bool {
            let __self_tag = ::core::intrinsics::discriminant_value(self);
            let __arg1_tag = ::core::intrinsics::discriminant_value(other);
            __self_tag == __arg1_tag
        }
    }
    #[automatically_derived]
    impl ::core::cmp::Eq for Anchor {
        #[inline]
        #[doc(hidden)]
        #[coverage(off)]
        fn assert_receiver_is_total_eq(&self) -> () {}
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
    pub enum PopupStyle {
        #[default]
        Minimal,
    }
    #[automatically_derived]
    impl ::core::fmt::Debug for PopupStyle {
        #[inline]
        fn fmt(&self, f: &mut ::core::fmt::Formatter) -> ::core::fmt::Result {
            ::core::fmt::Formatter::write_str(f, "Minimal")
        }
    }
    #[automatically_derived]
    impl ::core::default::Default for PopupStyle {
        #[inline]
        fn default() -> PopupStyle {
            Self::Minimal
        }
    }
    #[automatically_derived]
    impl ::core::clone::Clone for PopupStyle {
        #[inline]
        fn clone(&self) -> PopupStyle {
            *self
        }
    }
    #[automatically_derived]
    impl ::core::marker::Copy for PopupStyle {}
    #[automatically_derived]
    impl ::core::marker::StructuralPartialEq for PopupStyle {}
    #[automatically_derived]
    impl ::core::cmp::PartialEq for PopupStyle {
        #[inline]
        fn eq(&self, other: &PopupStyle) -> bool {
            true
        }
    }
    #[automatically_derived]
    impl ::core::cmp::Eq for PopupStyle {
        #[inline]
        #[doc(hidden)]
        #[coverage(off)]
        fn assert_receiver_is_total_eq(&self) -> () {}
    }
    impl fmt::Display for PopupStyle {
        fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
            f.write_str("minimal")
        }
    }
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
    #[automatically_derived]
    impl ::core::fmt::Debug for PopupBorder {
        #[inline]
        fn fmt(&self, f: &mut ::core::fmt::Formatter) -> ::core::fmt::Result {
            match self {
                PopupBorder::None => ::core::fmt::Formatter::write_str(f, "None"),
                PopupBorder::Single => ::core::fmt::Formatter::write_str(f, "Single"),
                PopupBorder::Double => ::core::fmt::Formatter::write_str(f, "Double"),
                PopupBorder::Rounded => ::core::fmt::Formatter::write_str(f, "Rounded"),
                PopupBorder::Solid => ::core::fmt::Formatter::write_str(f, "Solid"),
                PopupBorder::Shadow => ::core::fmt::Formatter::write_str(f, "Shadow"),
                PopupBorder::Custom(__self_0) => {
                    ::core::fmt::Formatter::debug_tuple_field1_finish(
                        f,
                        "Custom",
                        &__self_0,
                    )
                }
            }
        }
    }
    #[automatically_derived]
    impl ::core::default::Default for PopupBorder {
        #[inline]
        fn default() -> PopupBorder {
            Self::None
        }
    }
    #[automatically_derived]
    impl ::core::clone::Clone for PopupBorder {
        #[inline]
        fn clone(&self) -> PopupBorder {
            match self {
                PopupBorder::None => PopupBorder::None,
                PopupBorder::Single => PopupBorder::Single,
                PopupBorder::Double => PopupBorder::Double,
                PopupBorder::Rounded => PopupBorder::Rounded,
                PopupBorder::Solid => PopupBorder::Solid,
                PopupBorder::Shadow => PopupBorder::Shadow,
                PopupBorder::Custom(__self_0) => {
                    PopupBorder::Custom(::core::clone::Clone::clone(__self_0))
                }
            }
        }
    }
    #[automatically_derived]
    impl ::core::marker::StructuralPartialEq for PopupBorder {}
    #[automatically_derived]
    impl ::core::cmp::PartialEq for PopupBorder {
        #[inline]
        fn eq(&self, other: &PopupBorder) -> bool {
            let __self_tag = ::core::intrinsics::discriminant_value(self);
            let __arg1_tag = ::core::intrinsics::discriminant_value(other);
            __self_tag == __arg1_tag
                && match (self, other) {
                    (PopupBorder::Custom(__self_0), PopupBorder::Custom(__arg1_0)) => {
                        *__self_0 == *__arg1_0
                    }
                    _ => true,
                }
        }
    }
    #[automatically_derived]
    impl ::core::cmp::Eq for PopupBorder {
        #[inline]
        #[doc(hidden)]
        #[coverage(off)]
        fn assert_receiver_is_total_eq(&self) -> () {
            let _: ::core::cmp::AssertParamIsEq<Vec<String>>;
        }
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
    pub enum PopupAlign {
        #[default]
        Left,
        Center,
        Right,
    }
    #[automatically_derived]
    impl ::core::fmt::Debug for PopupAlign {
        #[inline]
        fn fmt(&self, f: &mut ::core::fmt::Formatter) -> ::core::fmt::Result {
            ::core::fmt::Formatter::write_str(
                f,
                match self {
                    PopupAlign::Left => "Left",
                    PopupAlign::Center => "Center",
                    PopupAlign::Right => "Right",
                },
            )
        }
    }
    #[automatically_derived]
    impl ::core::default::Default for PopupAlign {
        #[inline]
        fn default() -> PopupAlign {
            Self::Left
        }
    }
    #[automatically_derived]
    impl ::core::clone::Clone for PopupAlign {
        #[inline]
        fn clone(&self) -> PopupAlign {
            *self
        }
    }
    #[automatically_derived]
    impl ::core::marker::Copy for PopupAlign {}
    #[automatically_derived]
    impl ::core::marker::StructuralPartialEq for PopupAlign {}
    #[automatically_derived]
    impl ::core::cmp::PartialEq for PopupAlign {
        #[inline]
        fn eq(&self, other: &PopupAlign) -> bool {
            let __self_tag = ::core::intrinsics::discriminant_value(self);
            let __arg1_tag = ::core::intrinsics::discriminant_value(other);
            __self_tag == __arg1_tag
        }
    }
    #[automatically_derived]
    impl ::core::cmp::Eq for PopupAlign {
        #[inline]
        #[doc(hidden)]
        #[coverage(off)]
        fn assert_receiver_is_total_eq(&self) -> () {}
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
    #[serde(rename_all = "snake_case")]
    pub enum PopupSplit {
        #[default]
        Left,
        Right,
        Above,
        Below,
    }
    #[automatically_derived]
    impl ::core::fmt::Debug for PopupSplit {
        #[inline]
        fn fmt(&self, f: &mut ::core::fmt::Formatter) -> ::core::fmt::Result {
            ::core::fmt::Formatter::write_str(
                f,
                match self {
                    PopupSplit::Left => "Left",
                    PopupSplit::Right => "Right",
                    PopupSplit::Above => "Above",
                    PopupSplit::Below => "Below",
                },
            )
        }
    }
    #[automatically_derived]
    impl ::core::default::Default for PopupSplit {
        #[inline]
        fn default() -> PopupSplit {
            Self::Left
        }
    }
    #[doc(hidden)]
    #[allow(non_upper_case_globals, unused_attributes, unused_qualifications)]
    const _: () = {
        #[allow(unused_extern_crates, clippy::useless_attribute)]
        extern crate serde as _serde;
        #[automatically_derived]
        impl _serde::Serialize for PopupSplit {
            fn serialize<__S>(
                &self,
                __serializer: __S,
            ) -> _serde::__private::Result<__S::Ok, __S::Error>
            where
                __S: _serde::Serializer,
            {
                match *self {
                    PopupSplit::Left => {
                        _serde::Serializer::serialize_unit_variant(
                            __serializer,
                            "PopupSplit",
                            0u32,
                            "left",
                        )
                    }
                    PopupSplit::Right => {
                        _serde::Serializer::serialize_unit_variant(
                            __serializer,
                            "PopupSplit",
                            1u32,
                            "right",
                        )
                    }
                    PopupSplit::Above => {
                        _serde::Serializer::serialize_unit_variant(
                            __serializer,
                            "PopupSplit",
                            2u32,
                            "above",
                        )
                    }
                    PopupSplit::Below => {
                        _serde::Serializer::serialize_unit_variant(
                            __serializer,
                            "PopupSplit",
                            3u32,
                            "below",
                        )
                    }
                }
            }
        }
    };
    #[automatically_derived]
    impl ::core::clone::Clone for PopupSplit {
        #[inline]
        fn clone(&self) -> PopupSplit {
            *self
        }
    }
    #[automatically_derived]
    impl ::core::marker::Copy for PopupSplit {}
    #[automatically_derived]
    impl ::core::marker::StructuralPartialEq for PopupSplit {}
    #[automatically_derived]
    impl ::core::cmp::PartialEq for PopupSplit {
        #[inline]
        fn eq(&self, other: &PopupSplit) -> bool {
            let __self_tag = ::core::intrinsics::discriminant_value(self);
            let __arg1_tag = ::core::intrinsics::discriminant_value(other);
            __self_tag == __arg1_tag
        }
    }
    #[automatically_derived]
    impl ::core::cmp::Eq for PopupSplit {
        #[inline]
        #[doc(hidden)]
        #[coverage(off)]
        fn assert_receiver_is_total_eq(&self) -> () {}
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
    pub enum PopupSize {
        Fixed(u32),
        /// Between 0 and 1
        Percentage(f32),
    }
    #[automatically_derived]
    impl ::core::fmt::Debug for PopupSize {
        #[inline]
        fn fmt(&self, f: &mut ::core::fmt::Formatter) -> ::core::fmt::Result {
            match self {
                PopupSize::Fixed(__self_0) => {
                    ::core::fmt::Formatter::debug_tuple_field1_finish(
                        f,
                        "Fixed",
                        &__self_0,
                    )
                }
                PopupSize::Percentage(__self_0) => {
                    ::core::fmt::Formatter::debug_tuple_field1_finish(
                        f,
                        "Percentage",
                        &__self_0,
                    )
                }
            }
        }
    }
    #[automatically_derived]
    impl ::core::clone::Clone for PopupSize {
        #[inline]
        fn clone(&self) -> PopupSize {
            let _: ::core::clone::AssertParamIsClone<u32>;
            let _: ::core::clone::AssertParamIsClone<f32>;
            *self
        }
    }
    #[automatically_derived]
    impl ::core::marker::Copy for PopupSize {}
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
    #[automatically_derived]
    impl ::core::default::Default for WinOptions {
        #[inline]
        fn default() -> WinOptions {
            WinOptions {
                width: ::core::default::Default::default(),
                height: ::core::default::Default::default(),
                col: ::core::default::Default::default(),
                row: ::core::default::Default::default(),
                relative: ::core::default::Default::default(),
                buf_pos: ::core::default::Default::default(),
                win: ::core::default::Default::default(),
                anchor: ::core::default::Default::default(),
                focusable: ::core::default::Default::default(),
                external: ::core::default::Default::default(),
                zindex: ::core::default::Default::default(),
                style: ::core::default::Default::default(),
                border: ::core::default::Default::default(),
                title: ::core::default::Default::default(),
                title_pos: ::core::default::Default::default(),
                footer: ::core::default::Default::default(),
                footer_pos: ::core::default::Default::default(),
                noautocmd: ::core::default::Default::default(),
                fixed: ::core::default::Default::default(),
                hide: ::core::default::Default::default(),
                vertical: ::core::default::Default::default(),
                split: ::core::default::Default::default(),
            }
        }
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
    pub struct NeoPopup {
        pub win: NeoWindow,
        pub buf: NeoBuffer,
    }
    #[automatically_derived]
    impl ::core::clone::Clone for NeoPopup {
        #[inline]
        fn clone(&self) -> NeoPopup {
            let _: ::core::clone::AssertParamIsClone<NeoWindow>;
            let _: ::core::clone::AssertParamIsClone<NeoBuffer>;
            *self
        }
    }
    #[automatically_derived]
    impl ::core::marker::Copy for NeoPopup {}
    #[automatically_derived]
    impl ::core::fmt::Debug for NeoPopup {
        #[inline]
        fn fmt(&self, f: &mut ::core::fmt::Formatter) -> ::core::fmt::Result {
            ::core::fmt::Formatter::debug_struct_field2_finish(
                f,
                "NeoPopup",
                "win",
                &self.win,
                "buf",
                &&self.buf,
            )
        }
    }
    #[automatically_derived]
    impl ::core::marker::StructuralPartialEq for NeoPopup {}
    #[automatically_derived]
    impl ::core::cmp::PartialEq for NeoPopup {
        #[inline]
        fn eq(&self, other: &NeoPopup) -> bool {
            self.win == other.win && self.buf == other.buf
        }
    }
    #[automatically_derived]
    impl ::core::cmp::Eq for NeoPopup {
        #[inline]
        #[doc(hidden)]
        #[coverage(off)]
        fn assert_receiver_is_total_eq(&self) -> () {
            let _: ::core::cmp::AssertParamIsEq<NeoWindow>;
            let _: ::core::cmp::AssertParamIsEq<NeoBuffer>;
        }
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
    pub enum PopupLevel {
        Succes,
        Error,
        Neutral,
    }
    #[automatically_derived]
    impl ::core::clone::Clone for PopupLevel {
        #[inline]
        fn clone(&self) -> PopupLevel {
            *self
        }
    }
    #[automatically_derived]
    impl ::core::marker::Copy for PopupLevel {}
    #[automatically_derived]
    impl ::core::fmt::Debug for PopupLevel {
        #[inline]
        fn fmt(&self, f: &mut ::core::fmt::Formatter) -> ::core::fmt::Result {
            ::core::fmt::Formatter::write_str(
                f,
                match self {
                    PopupLevel::Succes => "Succes",
                    PopupLevel::Error => "Error",
                    PopupLevel::Neutral => "Neutral",
                },
            )
        }
    }
    #[automatically_derived]
    impl ::core::marker::StructuralPartialEq for PopupLevel {}
    #[automatically_derived]
    impl ::core::cmp::PartialEq for PopupLevel {
        #[inline]
        fn eq(&self, other: &PopupLevel) -> bool {
            let __self_tag = ::core::intrinsics::discriminant_value(self);
            let __arg1_tag = ::core::intrinsics::discriminant_value(other);
            __self_tag == __arg1_tag
        }
    }
    #[automatically_derived]
    impl ::core::cmp::Eq for PopupLevel {
        #[inline]
        #[doc(hidden)]
        #[coverage(off)]
        fn assert_receiver_is_total_eq(&self) -> () {}
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
        pub fn open(
            lua: &Lua,
            buf: NeoBuffer,
            enter: bool,
            config: WinOptions,
        ) -> LuaResult<Self> {
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
                    title: Some(
                        TextType::Tuples(
                            <[_]>::into_vec(
                                #[rustc_box]
                                ::alloc::boxed::Box::new([
                                    HLText::new(options.title, options.level.to_string()),
                                ]),
                            ),
                        ),
                    ),
                    title_pos: PopupAlign::Left,
                    ..Default::default()
                },
            )?;
            let close_popup = lua
                .create_function(move |lua, _: ()| popup_win.close(lua, true))?;
            NeoApi::delay(lua, options.duration.as_millis() as u32, close_popup)
        }
    }
}
mod window {
    #![allow(unused)]
    use crate::{neo_api::NeoApi, neo_api_types::{OptValueType, WinCursor}};
    use mlua::prelude::{IntoLua, Lua, LuaFunction, LuaResult};
    pub struct NeoWindow(u32);
    #[automatically_derived]
    impl ::core::clone::Clone for NeoWindow {
        #[inline]
        fn clone(&self) -> NeoWindow {
            let _: ::core::clone::AssertParamIsClone<u32>;
            *self
        }
    }
    #[automatically_derived]
    impl ::core::marker::Copy for NeoWindow {}
    #[automatically_derived]
    impl ::core::fmt::Debug for NeoWindow {
        #[inline]
        fn fmt(&self, f: &mut ::core::fmt::Formatter) -> ::core::fmt::Result {
            ::core::fmt::Formatter::debug_tuple_field1_finish(f, "NeoWindow", &&self.0)
        }
    }
    #[automatically_derived]
    impl ::core::marker::StructuralPartialEq for NeoWindow {}
    #[automatically_derived]
    impl ::core::cmp::PartialEq for NeoWindow {
        #[inline]
        fn eq(&self, other: &NeoWindow) -> bool {
            self.0 == other.0
        }
    }
    #[automatically_derived]
    impl ::core::cmp::Eq for NeoWindow {
        #[inline]
        #[doc(hidden)]
        #[coverage(off)]
        fn assert_receiver_is_total_eq(&self) -> () {
            let _: ::core::cmp::AssertParamIsEq<u32>;
        }
    }
    #[automatically_derived]
    impl ::core::hash::Hash for NeoWindow {
        #[inline]
        fn hash<__H: ::core::hash::Hasher>(&self, state: &mut __H) -> () {
            ::core::hash::Hash::hash(&self.0, state)
        }
    }
    impl NeoWindow {
        pub const CURRENT: Self = Self(0);
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
        /**
    Sets the (1,0)-indexed cursor position in the window. |api-indexing| This
    scrolls the window even if it is not the current one.

    Parameters: ~
      • Window handle, or 0 for current window
      • WinCursor
    */
        pub fn set_cursor(&self, lua: &Lua, cursor: WinCursor) -> LuaResult<()> {
            let lfn: LuaFunction = lua.load("vim.api.nvim_win_set_cursor").eval()?;
            lfn.call::<_, ()>((self.id(), cursor))
        }
        /**
    Gets the (1,0)-indexed, buffer-relative cursor position for a given window
    (different windows showing the same buffer have independent cursor
    positions). |api-indexing|

    Parameters: ~
      • {window}  Window handle, or 0 for current window

    See also: ~
      • |getcurpos()|
    */
        pub fn get_cursor(&self, lua: &Lua) -> LuaResult<WinCursor> {
            let lfn: LuaFunction = lua.load("vim.api.nvim_win_get_cursor").eval()?;
            lfn.call((self.id()))
        }
        /// Adds the namespace scope to the window.
        pub fn add_ns(&self, lua: &Lua, ns_id: u32) -> LuaResult<()> {
            let lfn: LuaFunction = lua.load("vim.api.nvim_win_add_ns").eval()?;
            lfn.call((self.id(), ns_id))
        }
        pub fn call<'a>(&self, lua: &'a Lua, cb: LuaFunction<'a>) -> LuaResult<()> {
            let lfn: LuaFunction = lua.load("vim.api.nvim_win_call").eval()?;
            lfn.call((self.id(), cb))
        }
        /**
    nvim_win_close({window}, {force})                           *nvim_win_close()*
    Closes the window (like |:close| with a |window-ID|).

    Attributes: ~
        not allowed when |textlock| is active
    */
        pub fn close(&self, lua: &Lua, force: bool) -> LuaResult<()> {
            let lfn: LuaFunction = lua.load("vim.api.nvim_win_close").eval()?;
            lfn.call::<_, ()>((self.id(), force))
        }
    }
}
mod fuzzy {
    #![allow(unused)]
    use mlua::Lua;
    use mlua::{
        prelude::{LuaResult, LuaValue},
        IntoLua,
    };
    use once_cell::sync::Lazy;
    use std::cmp::Ordering;
    use std::fmt;
    use std::future::Future;
    use std::io::BufReader;
    use std::process::ExitStatus;
    use std::{collections::HashMap, path::{Path, PathBuf}};
    use std::{process::Stdio, time::Instant};
    use tokio::io::{self, AsyncWriteExt};
    use tokio::process::*;
    use tokio::runtime::Runtime;
    use tokio::sync::RwLock;
    use tokio::sync::{Mutex, RwLockReadGuard, TryLockError};
    use tokio::{fs, join};
    use crate::{
        callback, AutoCmdCbEvent, AutoCmdEvent, AutoCmdGroup, BufferDeleteOpts, CmdOpts,
        FastLock, HLOpts, Mode, NeoApi, NeoBuffer, NeoPopup, NeoTheme, NeoWindow,
        PopupBorder, PopupRelative, PopupSize, PopupSplit, PopupStyle, TextType,
    };
    const GRP_FUZZY_SELECT: &str = "NeoFuzzySelect";
    const GRP_FUZZY_LETTER: &str = "NeoFuzzyLetter";
    struct FuzzyContainer {
        all_lines: RwLock<String>,
        cached_lines: RwLock<Vec<String>>,
        rt: Runtime,
        fuzzy: RwLock<Option<NeoFuzzy>>,
        query_meta: RwLock<QueryMeta>,
        preview: RwLock<Vec<String>>,
    }
    pub trait FuzzyConfig: Send + Sync {
        fn cwd(&self, lua: &Lua) -> PathBuf;
        fn search_type(&self) -> FilesSearch;
        fn on_enter(&self, lua: &Lua, item: PathBuf);
    }
    impl fmt::Debug for dyn FuzzyConfig {
        fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
            Ok(())
        }
    }
    struct QueryMeta {
        last_search: String,
        update_results: bool,
    }
    #[automatically_derived]
    impl ::core::fmt::Debug for QueryMeta {
        #[inline]
        fn fmt(&self, f: &mut ::core::fmt::Formatter) -> ::core::fmt::Result {
            ::core::fmt::Formatter::debug_struct_field2_finish(
                f,
                "QueryMeta",
                "last_search",
                &self.last_search,
                "update_results",
                &&self.update_results,
            )
        }
    }
    unsafe impl Send for FuzzyContainer {}
    unsafe impl Sync for FuzzyContainer {}
    static CONTAINER: Lazy<FuzzyContainer> = Lazy::new(|| FuzzyContainer {
        all_lines: RwLock::new(String::new()),
        cached_lines: RwLock::new(::alloc::vec::Vec::new()),
        fuzzy: RwLock::new(None),
        rt: tokio::runtime::Runtime::new().unwrap(),
        query_meta: RwLock::new(QueryMeta {
            update_results: false,
            last_search: "".to_string(),
        }),
        preview: RwLock::new(Vec::new()),
    });
    pub struct NeoFuzzy {
        pub pop_cmd: NeoPopup,
        pub pop_out: NeoPopup,
        pub pop_preview: NeoPopup,
        pub cwd: PathBuf,
        pub args: Vec<String>,
        pub cmd: String,
        pub selected_idx: usize,
        pub ns_id: u32,
        pub config: Box<dyn FuzzyConfig>,
    }
    #[automatically_derived]
    impl ::core::fmt::Debug for NeoFuzzy {
        #[inline]
        fn fmt(&self, f: &mut ::core::fmt::Formatter) -> ::core::fmt::Result {
            let names: &'static _ = &[
                "pop_cmd",
                "pop_out",
                "pop_preview",
                "cwd",
                "args",
                "cmd",
                "selected_idx",
                "ns_id",
                "config",
            ];
            let values: &[&dyn ::core::fmt::Debug] = &[
                &self.pop_cmd,
                &self.pop_out,
                &self.pop_preview,
                &self.cwd,
                &self.args,
                &self.cmd,
                &self.selected_idx,
                &self.ns_id,
                &&self.config,
            ];
            ::core::fmt::Formatter::debug_struct_fields_finish(
                f,
                "NeoFuzzy",
                names,
                values,
            )
        }
    }
    pub enum FilesSearch {
        FileOnly,
        DirOnly,
        All,
    }
    pub enum Move {
        Up,
        Down,
    }
    impl NeoFuzzy {
        pub fn add_hl_groups(lua: &Lua) -> LuaResult<()> {
            NeoTheme::set_hl(
                lua,
                0,
                GRP_FUZZY_SELECT,
                HLOpts {
                    fg: Some("#39E75F".to_string()),
                    bold: true,
                    ..Default::default()
                },
            )?;
            NeoTheme::set_hl(
                lua,
                0,
                GRP_FUZZY_SELECT,
                HLOpts {
                    fg: Some("#CEFAD0".to_string()),
                    ..Default::default()
                },
            )?;
            Ok(())
        }
        pub fn add_keymaps(&self, lua: &Lua) -> LuaResult<()> {
            let buf = self.pop_cmd.buf;
            buf.set_keymap(
                lua,
                Mode::Insert,
                "<Up>",
                lua.create_async_function(|lua, _: ()| move_selection(lua, Move::Up))?,
            )?;
            buf.set_keymap(
                lua,
                Mode::Insert,
                "<Down>",
                lua.create_async_function(|lua, _: ()| move_selection(lua, Move::Down))?,
            )?;
            buf.set_keymap(
                lua,
                Mode::Insert,
                "<Esc>",
                lua.create_function(close_fuzzy)?,
            )?;
            buf.set_keymap(
                lua,
                Mode::Insert,
                "<Enter>",
                lua.create_async_function(Self::open_item)?,
            )
        }
        pub async fn open_item(lua: &Lua, _: ()) -> LuaResult<()> {
            let mut fuzzy = CONTAINER.fuzzy.write().await;
            if let Some(fuzzy) = fuzzy.as_mut() {
                let cached_lines = CONTAINER.cached_lines.read().await;
                let selected = fuzzy.cwd.join(cached_lines[fuzzy.selected_idx].as_str());
                NeoWindow::CURRENT.close(lua, true)?;
                fuzzy.config.on_enter(lua, selected);
            }
            Ok(())
        }
        pub async fn files(lua: &Lua, config: Box<dyn FuzzyConfig>) -> LuaResult<()> {
            Self::add_hl_groups(lua)?;
            let ns_id = NeoTheme::create_namespace(lua, "NeoFuzzy")?;
            NeoTheme::set_hl_ns(lua, ns_id)?;
            let ui = &NeoApi::list_uis(lua)?[0];
            let pop_cmd_row = ui.height - 6;
            let pop_cmd_col = 4;
            let pop_cmd_height = 1;
            let pop_cmd_width = if ui.width % 2 == 0 {
                ui.width - 8
            } else {
                ui.width - 9
            };
            let out_bat_height = (pop_cmd_row - 4);
            let out_width = pop_cmd_width / 2;
            let out_bat_row = 2;
            let out_col = pop_cmd_col;
            let bat_width = out_width - 2;
            let bat_col = pop_cmd_col + out_width + 2;
            let pop_cmd = NeoPopup::open(
                lua,
                NeoBuffer::create(lua, false, true)?,
                true,
                crate::WinOptions {
                    width: Some(PopupSize::Fixed(pop_cmd_width)),
                    height: Some(PopupSize::Fixed(pop_cmd_height)),
                    row: Some(PopupSize::Fixed(pop_cmd_row)),
                    col: Some(PopupSize::Fixed(pop_cmd_col)),
                    relative: PopupRelative::Editor,
                    border: PopupBorder::Single,
                    style: Some(PopupStyle::Minimal),
                    title: Some(TextType::String("Search for directory".to_string())),
                    ..Default::default()
                },
            )?;
            let pop_out = NeoPopup::open(
                lua,
                NeoBuffer::create(lua, false, true)?,
                false,
                crate::WinOptions {
                    width: Some(PopupSize::Fixed(out_width)),
                    height: Some(PopupSize::Fixed(out_bat_height)),
                    row: Some(PopupSize::Fixed(out_bat_row)),
                    col: Some(PopupSize::Fixed(out_col)),
                    relative: PopupRelative::Editor,
                    border: crate::PopupBorder::Single,
                    focusable: Some(false),
                    style: Some(PopupStyle::Minimal),
                    ..Default::default()
                },
            )?;
            let pop_preview = NeoPopup::open(
                lua,
                NeoBuffer::create(lua, false, true)?,
                false,
                crate::WinOptions {
                    width: Some(PopupSize::Fixed(bat_width)),
                    height: Some(PopupSize::Fixed(out_bat_height)),
                    row: Some(PopupSize::Fixed(out_bat_row)),
                    col: Some(PopupSize::Fixed(bat_col)),
                    relative: PopupRelative::Editor,
                    border: crate::PopupBorder::Single,
                    focusable: Some(false),
                    style: Some(PopupStyle::Minimal),
                    ..Default::default()
                },
            )?;
            pop_cmd.buf.set_current(lua)?;
            NeoApi::set_insert_mode(lua, true)?;
            let group = NeoApi::create_augroup(lua, "neo-fuzzy", false)?;
            let callback = lua.create_async_function(aucmd_text_changed)?;
            let test = NeoApi::schedule_wrap(lua, callback)?;
            NeoApi::create_autocmd(
                lua,
                &[AutoCmdEvent::TextChangedI],
                crate::AutoCmdOpts {
                    callback: test,
                    buffer: Some(pop_cmd.buf.id()),
                    group: Some(AutoCmdGroup::Integer(group)),
                    pattern: ::alloc::vec::Vec::new(),
                    once: false,
                    desc: None,
                },
            )?;
            NeoApi::create_autocmd(
                lua,
                &[AutoCmdEvent::BufLeave],
                crate::AutoCmdOpts {
                    callback: lua.create_function(aucmd_close_fuzzy)?,
                    buffer: Some(pop_cmd.buf.id()),
                    group: Some(AutoCmdGroup::Integer(group)),
                    pattern: ::alloc::vec::Vec::new(),
                    once: true,
                    desc: None,
                },
            )?;
            let cmd = "fd".to_string();
            let cwd = config.cwd(lua);
            let args = match config.search_type() {
                FilesSearch::All => ::alloc::vec::Vec::new(),
                FilesSearch::DirOnly => {
                    <[_]>::into_vec(
                        #[rustc_box]
                        ::alloc::boxed::Box::new([
                            "--type".to_string(),
                            "directory".to_string(),
                        ]),
                    )
                }
                FilesSearch::FileOnly => {
                    <[_]>::into_vec(
                        #[rustc_box]
                        ::alloc::boxed::Box::new([
                            "--type".to_string(),
                            "file".to_string(),
                        ]),
                    )
                }
            };
            let mut fuzzy = NeoFuzzy {
                pop_cmd,
                pop_out,
                pop_preview,
                cwd,
                args,
                cmd,
                selected_idx: 0,
                ns_id,
                config,
            };
            fuzzy.add_keymaps(lua)?;
            let rt = &CONTAINER.rt;
            let cwd = fuzzy.cwd.clone();
            let cmd = fuzzy.cmd.clone();
            let args = fuzzy.args.clone();
            rt.spawn(async move {
                exec_search(&cwd, cmd, args, "").await;
                prepare_preview(&cwd, 0).await;
            });
            let mut container = CONTAINER.fuzzy.write().await;
            *container = Some(fuzzy);
            drop(container);
            let interval = lua.create_async_function(interval_write_out)?;
            NeoApi::start_interval(lua, "fuzzy", 32, interval)?;
            Ok(())
        }
        pub fn fuzzy_grep(cwd: &Path, text: String) {}
        fn add_preview_highlight(&self, lua: &Lua, preview: &[String]) -> LuaResult<()> {
            self.pop_preview.buf.clear_namespace(lua, self.ns_id as i32, 0, -1)?;
            for (i, item_name) in preview.iter().enumerate() {
                if item_name.ends_with("/") {
                    self.pop_preview
                        .buf
                        .add_highlight(lua, self.ns_id as i32, "Directory", i, 0, -1)?;
                }
                if item_name.starts_with("> Empty") {
                    self.pop_preview
                        .buf
                        .add_highlight(lua, self.ns_id as i32, "Comment", i, 0, -1)?;
                }
            }
            Ok(())
        }
        fn add_out_highlight(&self, lua: &Lua) -> LuaResult<()> {
            self.pop_out.buf.clear_namespace(lua, self.ns_id as i32, 0, -1)?;
            self.pop_out
                .buf
                .add_highlight(
                    lua,
                    self.ns_id as i32,
                    "NeoFuzzySelect",
                    self.selected_idx,
                    0,
                    -1,
                )?;
            Ok(())
        }
    }
    async fn exec_search(
        cwd: &Path,
        cmd: String,
        args: Vec<String>,
        search_query: &str,
    ) -> io::Result<()> {
        let sanitized = search_query.replace('/', "\\/").replace('.', "\\.");
        let has_lines = !CONTAINER.all_lines.read().await.is_empty();
        async fn sort_lines(lines: &str, search_query: &str) {
            let mut new_lines = Vec::new();
            for line in lines.lines() {
                let score = levenshtein(&search_query, line);
                new_lines.push((score, line.to_string()));
            }
            new_lines.sort_by_key(|k| k.0);
            let new_lines = new_lines.into_iter().map(|k| k.1);
            let mut cached_lines = CONTAINER.cached_lines.write().await;
            cached_lines.clear();
            for (i, line) in new_lines.enumerate() {
                cached_lines.push(line);
                if 300 == i + 1 {
                    break;
                }
            }
            let mut query = CONTAINER.query_meta.write().await;
            query.last_search = search_query.to_string();
        }
        if !has_lines {
            let out = Command::new(cmd).current_dir(cwd).args(args).output().await?;
            if out.status.success() {
                let mut lines = CONTAINER.all_lines.write().await;
                *lines = String::from_utf8_lossy(&out.stdout).to_string();
                sort_lines(&lines, "").await;
            }
        } else {
            let mut regex = String::from(".*");
            for char in sanitized.chars() {
                if char.is_lowercase() {
                    regex
                        .push_str(
                            &{
                                let res = ::alloc::fmt::format(
                                    format_args!("[{0}{1}]", char.to_uppercase(), char),
                                );
                                res
                            },
                        );
                } else {
                    regex.push(char);
                }
                if char != '\\' {
                    regex.push_str(".*");
                }
            }
            let mut rg_proc = Command::new("rg")
                .args(["--regexp", &regex])
                .stdin(Stdio::piped())
                .stdout(Stdio::piped())
                .stderr(Stdio::piped())
                .spawn()?;
            let mut stdin = rg_proc.stdin.take().unwrap();
            let query_len = search_query.len();
            tokio::spawn(async move {
                let lines = CONTAINER.all_lines.read().await;
                stdin.write_all(lines.as_bytes()).await.unwrap();
                drop(stdin);
            });
            let out = rg_proc.wait_with_output().await?;
            if out.status.success() {
                let lines = String::from_utf8_lossy(&out.stdout);
                sort_lines(&lines, search_query).await;
            }
        }
        Ok(())
    }
    async fn prepare_preview(cwd: &Path, selected_idx: usize) -> io::Result<()> {
        let cached_lines = CONTAINER.cached_lines.read().await;
        let path: PathBuf = cwd.join(cached_lines[selected_idx].as_str());
        drop(cached_lines);
        let mut items = Vec::new();
        let mut dir = fs::read_dir(path).await?;
        while let Some(item) = dir.next_entry().await? {
            if let Ok(file_type) = item.file_type().await {
                let name = item.file_name().to_string_lossy().to_string();
                if file_type.is_dir() {
                    items
                        .push({
                            let res = ::alloc::fmt::format(format_args!("{0}/", name));
                            res
                        });
                } else {
                    items.push(name);
                }
            }
        }
        items
            .sort_by(|a, b| {
                if a.ends_with('/') == b.ends_with('/') {
                    a.cmp(&b)
                } else if a.ends_with('/') {
                    Ordering::Less
                } else {
                    Ordering::Greater
                }
            });
        if items.is_empty() {
            items.push("> Empty directory".to_string());
        }
        let mut preview = CONTAINER.preview.write().await;
        *preview = items;
        drop(preview);
        let mut query = CONTAINER.query_meta.write().await;
        query.update_results = true;
        Ok(())
    }
    async fn interval_write_out(lua: &Lua, _: ()) -> LuaResult<()> {
        let query_meta = CONTAINER.query_meta.try_read();
        let fuzzy = CONTAINER.fuzzy.try_read();
        if query_meta.is_err() || fuzzy.is_err() {
            return Ok(());
        }
        let query = query_meta.unwrap();
        if query.update_results {
            if let Some(fuzzy) = fuzzy.unwrap().as_ref() {
                if let Ok(lines) = CONTAINER.cached_lines.try_read() {
                    fuzzy.pop_out.buf.set_lines(lua, 0, -1, false, &lines);
                    fuzzy.add_out_highlight(lua);
                }
                if let Ok(preview) = CONTAINER.preview.try_read() {
                    fuzzy.pop_preview.buf.set_lines(lua, 0, -1, false, &preview);
                    fuzzy.add_preview_highlight(lua, &preview);
                }
            }
        }
        Ok(())
    }
    async fn move_selection(lua: &Lua, move_sel: Move) -> LuaResult<()> {
        let mut fuzzy = CONTAINER.fuzzy.write().await;
        if let Some(fuzzy) = fuzzy.as_mut() {
            let len = fuzzy.pop_out.buf.line_count(lua)?;
            let mut jump_line = false;
            match move_sel {
                Move::Up => {
                    if 0 < fuzzy.selected_idx {
                        fuzzy.selected_idx -= 1;
                        jump_line = true;
                    }
                }
                Move::Down => {
                    if fuzzy.selected_idx + 1 < len {
                        fuzzy.selected_idx += 1;
                        jump_line = true;
                    }
                }
            }
            if jump_line {
                let sel_idx = fuzzy.selected_idx;
                let cwd = fuzzy.cwd.clone();
                fuzzy
                    .pop_out
                    .win
                    .call(
                        lua,
                        lua
                            .create_function(move |lua, _: ()| {
                                NeoApi::cmd(
                                    lua,
                                    CmdOpts {
                                        cmd: "normal",
                                        bang: true,
                                        args: &[
                                            &{
                                                let res = ::alloc::fmt::format(
                                                    format_args!("{0}G", sel_idx + 1),
                                                );
                                                res
                                            },
                                        ],
                                    },
                                )
                            })?,
                    )?;
                let rt = &CONTAINER.rt;
                rt.spawn(async move {
                    prepare_preview(&cwd, sel_idx).await;
                });
            }
        }
        Ok(())
    }
    fn close_fuzzy(lua: &Lua, _: ()) -> LuaResult<()> {
        NeoWindow::CURRENT.close(lua, true)
    }
    fn aucmd_close_fuzzy(lua: &Lua, ev: AutoCmdCbEvent) -> LuaResult<()> {
        let mut container = CONTAINER.fuzzy.blocking_write();
        let buffer = NeoBuffer::new(ev.buf.unwrap());
        if let Some(fuzzy) = container.as_ref() {
            fuzzy.pop_out.win.close(lua, false)?;
            fuzzy.pop_cmd.win.close(lua, false)?;
            fuzzy.pop_preview.win.close(lua, false)?;
        }
        NeoApi::stop_interval(lua, "fuzzy")?;
        *container = None;
        NeoApi::set_insert_mode(lua, false)
    }
    async fn aucmd_text_changed(lua: &Lua, ev: AutoCmdCbEvent) -> LuaResult<()> {
        let search_query = NeoApi::get_current_line(lua)?;
        let rt = &CONTAINER.rt;
        rt.spawn(async move {
            let mut fuzzy = CONTAINER.fuzzy.write().await;
            let cmd;
            let cwd;
            let args;
            if let Some(fuzzy) = fuzzy.as_mut() {
                fuzzy.selected_idx = 0;
                cwd = fuzzy.cwd.clone();
                cmd = fuzzy.cmd.clone();
                args = fuzzy.args.clone();
            } else {
                return;
            }
            drop(fuzzy);
            exec_search(&cwd, cmd, args, &search_query).await;
            prepare_preview(&cwd, 0).await;
        });
        Ok(())
    }
    pub fn levenshtein(a: &str, b: &str) -> usize {
        let mut result = 0;
        if a == b {
            return result;
        }
        let length_a = a.len();
        let length_b = b.len();
        if length_a == 0 {
            return length_b;
        } else if length_b == 0 {
            return length_a;
        }
        let mut cache: Vec<usize> = (1..).take(length_a).collect();
        let mut distance_a;
        let mut distance_b;
        for (index_b, code_b) in b.chars().enumerate() {
            result = index_b;
            distance_a = index_b;
            for (index_a, code_a) in a.chars().enumerate() {
                distance_b = if code_a == code_b { distance_a } else { distance_a + 1 };
                distance_a = cache[index_a];
                result = if distance_a > result {
                    if distance_b > result { result + 1 } else { distance_b }
                } else if distance_b > distance_a {
                    distance_a + 1
                } else {
                    distance_b
                };
                cache[index_a] = result;
            }
        }
        result
    }
}
mod theme {
    use crate::mlua::prelude::{
        Lua, LuaFunction, LuaResult, LuaSerializeOptions, LuaValue,
    };
    use mlua::{IntoLua, LuaSerdeExt};
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
        pub force: bool,
    }
    #[automatically_derived]
    impl ::core::clone::Clone for HLOpts {
        #[inline]
        fn clone(&self) -> HLOpts {
            HLOpts {
                fg: ::core::clone::Clone::clone(&self.fg),
                bg: ::core::clone::Clone::clone(&self.bg),
                sp: ::core::clone::Clone::clone(&self.sp),
                blend: ::core::clone::Clone::clone(&self.blend),
                bold: ::core::clone::Clone::clone(&self.bold),
                standout: ::core::clone::Clone::clone(&self.standout),
                underline: ::core::clone::Clone::clone(&self.underline),
                undercurl: ::core::clone::Clone::clone(&self.undercurl),
                underdouble: ::core::clone::Clone::clone(&self.underdouble),
                underdotted: ::core::clone::Clone::clone(&self.underdotted),
                underdashed: ::core::clone::Clone::clone(&self.underdashed),
                strikethrough: ::core::clone::Clone::clone(&self.strikethrough),
                italic: ::core::clone::Clone::clone(&self.italic),
                reverse: ::core::clone::Clone::clone(&self.reverse),
                nocombine: ::core::clone::Clone::clone(&self.nocombine),
                link: ::core::clone::Clone::clone(&self.link),
                default: ::core::clone::Clone::clone(&self.default),
                ctermfg: ::core::clone::Clone::clone(&self.ctermfg),
                ctermbg: ::core::clone::Clone::clone(&self.ctermbg),
                force: ::core::clone::Clone::clone(&self.force),
            }
        }
    }
    #[automatically_derived]
    impl ::core::fmt::Debug for HLOpts {
        #[inline]
        fn fmt(&self, f: &mut ::core::fmt::Formatter) -> ::core::fmt::Result {
            let names: &'static _ = &[
                "fg",
                "bg",
                "sp",
                "blend",
                "bold",
                "standout",
                "underline",
                "undercurl",
                "underdouble",
                "underdotted",
                "underdashed",
                "strikethrough",
                "italic",
                "reverse",
                "nocombine",
                "link",
                "default",
                "ctermfg",
                "ctermbg",
                "force",
            ];
            let values: &[&dyn ::core::fmt::Debug] = &[
                &self.fg,
                &self.bg,
                &self.sp,
                &self.blend,
                &self.bold,
                &self.standout,
                &self.underline,
                &self.undercurl,
                &self.underdouble,
                &self.underdotted,
                &self.underdashed,
                &self.strikethrough,
                &self.italic,
                &self.reverse,
                &self.nocombine,
                &self.link,
                &self.default,
                &self.ctermfg,
                &self.ctermbg,
                &&self.force,
            ];
            ::core::fmt::Formatter::debug_struct_fields_finish(
                f,
                "HLOpts",
                names,
                values,
            )
        }
    }
    #[automatically_derived]
    impl ::core::default::Default for HLOpts {
        #[inline]
        fn default() -> HLOpts {
            HLOpts {
                fg: ::core::default::Default::default(),
                bg: ::core::default::Default::default(),
                sp: ::core::default::Default::default(),
                blend: ::core::default::Default::default(),
                bold: ::core::default::Default::default(),
                standout: ::core::default::Default::default(),
                underline: ::core::default::Default::default(),
                undercurl: ::core::default::Default::default(),
                underdouble: ::core::default::Default::default(),
                underdotted: ::core::default::Default::default(),
                underdashed: ::core::default::Default::default(),
                strikethrough: ::core::default::Default::default(),
                italic: ::core::default::Default::default(),
                reverse: ::core::default::Default::default(),
                nocombine: ::core::default::Default::default(),
                link: ::core::default::Default::default(),
                default: ::core::default::Default::default(),
                ctermfg: ::core::default::Default::default(),
                ctermbg: ::core::default::Default::default(),
                force: ::core::default::Default::default(),
            }
        }
    }
    #[doc(hidden)]
    #[allow(non_upper_case_globals, unused_attributes, unused_qualifications)]
    const _: () = {
        #[allow(unused_extern_crates, clippy::useless_attribute)]
        extern crate serde as _serde;
        #[automatically_derived]
        impl _serde::Serialize for HLOpts {
            fn serialize<__S>(
                &self,
                __serializer: __S,
            ) -> _serde::__private::Result<__S::Ok, __S::Error>
            where
                __S: _serde::Serializer,
            {
                let mut __serde_state = _serde::Serializer::serialize_struct(
                    __serializer,
                    "HLOpts",
                    false as usize + 1 + 1 + 1 + 1 + 1 + 1 + 1 + 1 + 1 + 1 + 1 + 1 + 1
                        + 1 + 1 + 1 + 1 + 1 + 1 + 1,
                )?;
                _serde::ser::SerializeStruct::serialize_field(
                    &mut __serde_state,
                    "fg",
                    &self.fg,
                )?;
                _serde::ser::SerializeStruct::serialize_field(
                    &mut __serde_state,
                    "bg",
                    &self.bg,
                )?;
                _serde::ser::SerializeStruct::serialize_field(
                    &mut __serde_state,
                    "sp",
                    &self.sp,
                )?;
                _serde::ser::SerializeStruct::serialize_field(
                    &mut __serde_state,
                    "blend",
                    &self.blend,
                )?;
                _serde::ser::SerializeStruct::serialize_field(
                    &mut __serde_state,
                    "bold",
                    &self.bold,
                )?;
                _serde::ser::SerializeStruct::serialize_field(
                    &mut __serde_state,
                    "standout",
                    &self.standout,
                )?;
                _serde::ser::SerializeStruct::serialize_field(
                    &mut __serde_state,
                    "underline",
                    &self.underline,
                )?;
                _serde::ser::SerializeStruct::serialize_field(
                    &mut __serde_state,
                    "undercurl",
                    &self.undercurl,
                )?;
                _serde::ser::SerializeStruct::serialize_field(
                    &mut __serde_state,
                    "underdouble",
                    &self.underdouble,
                )?;
                _serde::ser::SerializeStruct::serialize_field(
                    &mut __serde_state,
                    "underdotted",
                    &self.underdotted,
                )?;
                _serde::ser::SerializeStruct::serialize_field(
                    &mut __serde_state,
                    "underdashed",
                    &self.underdashed,
                )?;
                _serde::ser::SerializeStruct::serialize_field(
                    &mut __serde_state,
                    "strikethrough",
                    &self.strikethrough,
                )?;
                _serde::ser::SerializeStruct::serialize_field(
                    &mut __serde_state,
                    "italic",
                    &self.italic,
                )?;
                _serde::ser::SerializeStruct::serialize_field(
                    &mut __serde_state,
                    "reverse",
                    &self.reverse,
                )?;
                _serde::ser::SerializeStruct::serialize_field(
                    &mut __serde_state,
                    "nocombine",
                    &self.nocombine,
                )?;
                _serde::ser::SerializeStruct::serialize_field(
                    &mut __serde_state,
                    "link",
                    &self.link,
                )?;
                _serde::ser::SerializeStruct::serialize_field(
                    &mut __serde_state,
                    "default",
                    &self.default,
                )?;
                _serde::ser::SerializeStruct::serialize_field(
                    &mut __serde_state,
                    "ctermfg",
                    &self.ctermfg,
                )?;
                _serde::ser::SerializeStruct::serialize_field(
                    &mut __serde_state,
                    "ctermbg",
                    &self.ctermbg,
                )?;
                _serde::ser::SerializeStruct::serialize_field(
                    &mut __serde_state,
                    "force",
                    &self.force,
                )?;
                _serde::ser::SerializeStruct::end(__serde_state)
            }
        }
    };
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
        pub fn set_hl(
            lua: &Lua,
            ns_id: u32,
            group_name: &str,
            opts: HLOpts,
        ) -> LuaResult<()> {
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
}
mod traits {
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
}
pub use buffer::*;
pub use callback::*;
pub use neo_api::*;
pub use neo_api_types::*;
pub use popup::*;
pub use window::*;
pub use fuzzy::*;
pub use theme::*;
pub use traits::*;
pub use mlua;
