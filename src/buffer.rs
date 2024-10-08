#![allow(unused)]
use crate::neo_api::NeoApi;
use crate::neo_api_types::{ExtmarkOpts, OptValueType};
use crate::{BufferDeleteOpts, FileTypeMatch, KeymapOpts, Mode};
use mlua::prelude::{IntoLua, Lua, LuaError, LuaFunction, LuaResult, LuaTable, LuaValue};
use mlua::FromLua;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Default)]
pub struct NeoBuffer(u32);

impl FromLua for NeoBuffer {
    fn from_lua(value: LuaValue, lua: &Lua) -> LuaResult<Self> {
        match value {
            LuaValue::Integer(num) => Ok(NeoBuffer::from_id(num as u32)),
            _ => Err(LuaError::FromLuaConversionError {
                from: "LuaValue",
                to: "NeoBuffer",
                message: Some("Not a number".to_string()),
            }),
        }
    }
}

impl IntoLua for NeoBuffer {
    fn into_lua(self, lua: &Lua) -> LuaResult<LuaValue> {
        Ok(LuaValue::Integer(self.0 as i64))
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
    pub fn create(lua: &Lua, listed: bool, scratch: bool) -> LuaResult<Self> {
        let lfn: LuaFunction = lua.load("vim.api.nvim_create_buf").eval()?;
        let buf_id: u32 = lfn.call((listed, scratch))?;

        if buf_id == 0 {
            return Err(LuaError::RuntimeError("Buffer not created".to_string()));
        }

        Ok(NeoBuffer::from_id(buf_id))
    }

    //pub fn exists(&self, lua: &Lua) -> LuaResult<bool> {
    //let lfn: LuaFunction = lua.load("vim.fn.buflisted").eval()?;

    //lfn.call(self.id())
    //}

    pub fn bufadd(lua: &Lua, path: &str) -> LuaResult<Self> {
        let lfn: LuaFunction = lua.load("vim.fn.bufadd").eval()?;

        lfn.call(path)
    }

    /// This will create a buffer rust object, but not the corresponding lua object. 
    /// For that use the `NeoBuffer::create` function
    pub fn from_id(id: u32) -> Self {
        Self(id)
    }

    pub fn id(&self) -> u32 {
        self.0
    }

    pub fn get_current_buf(lua: &Lua) -> LuaResult<NeoBuffer> {
        let lfn: LuaFunction = lua.load("vim.api.nvim_get_current_buf").eval()?;

        lfn.call(())
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
        rhs: LuaFunction,
    ) -> LuaResult<()> {
        NeoApi::set_keymap(lua, mode, lhs, rhs, self.keymap_opts(true))
    }

    pub fn set_name(&self, lua: &Lua, path: &str) -> LuaResult<()> {
        let lfn: LuaFunction = lua.load("vim.api.nvim_buf_set_name").eval()?;

        lfn.call((self.id(), path))
    }

    /**
    Sets the current buffer.

    Attributes: ~
        not allowed when |textlock| is active or in the |cmdwin|
    */
    pub fn set_current(&self, lua: &Lua) -> LuaResult<()> {
        let lfn: LuaFunction = lua.load("vim.api.nvim_set_current_buf").eval()?;

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

        // Bug in nvim API, it won't allow opts not being passed, so create an empty table
        lfn.call((self.id(), opts))
    }

    /**
    Sets the value of an option. The behavior of this function matches that of
    |:set|: for global-local options, both the global and local value are set
    unless otherwise specified with {scope}.

    Parameters: ~
      • {name}   Option name
      • {value}  New option value
    */
    pub fn set_option_value<V: IntoLua>(
        &self,
        lua: &Lua,
        key: &str,
        value: V,
    ) -> LuaResult<()> {
        NeoApi::set_option_value(lua, key, value, OptValueType::Buffer(*self))
    }

    pub fn get_option_value<'a, V: FromLua>(&self, lua: &'a Lua, key: &str) -> LuaResult<V> {
        NeoApi::get_option_value(lua, key, OptValueType::Buffer(*self))
    }

    pub fn get_treesitter_lang(&self, lua: &Lua, ft: &str) -> LuaResult<Option<String>> {
        let lfn_gl: LuaFunction = lua.load("vim.treesitter.language.get_lang").eval()?;
        lfn_gl.call(ft)
    }

    pub fn start_treesitter(&self, lua: &Lua, lang: &str) -> LuaResult<()> {
        let lfn: LuaFunction = lua.load("vim.treesitter.start").eval()?;
        lfn.call((self.id(), lang))
    }

    pub fn stop_treesitter(&self, lua: &Lua) -> LuaResult<()> {
        let lfn: LuaFunction = lua.load("vim.treesitter.stop").eval()?;
        lfn.call(self.id())
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
    pub fn set_lines<T: AsRef<str>>(
        &self,
        lua: &Lua,
        start: i32,
        end: i32,
        strict_indexing: bool,
        lines: &[T],
    ) -> mlua::Result<()> {
        let lfn: LuaFunction = lua.load("vim.api.nvim_buf_set_lines").eval()?;
        let lines: Vec<_> = lines.iter().map(AsRef::as_ref).collect();

        lfn.call((self.id(), start, end, strict_indexing, lines))
    }

    pub fn set_text(
        &self, lua: &Lua,
        start_row: u32, start_col: u32, end_row: u32, end_col: u32) -> LuaResult<()> {
        let lfn: LuaFunction = lua.load("vim.api.nvim_buf_set_text").eval()?;

        lfn.call((self.id(), start_row, start_col, end_row, end_col))
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

    pub fn call(&self, lua: &Lua, cb: LuaFunction) -> LuaResult<()> {
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
    pub fn set_extmarks(
        &self,
        lua: &Lua,
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
