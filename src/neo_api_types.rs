use crate::buffer::NeoBuffer;
use crate::window::NeoWindow;
use macros::{FromTable, IntoEnum, IntoEnumSC, IntoTable};
use mlua::prelude::*;
use std::fmt::{self, Display};

#[derive(Debug, Clone, Copy, PartialEq, Eq, IntoEnumSC)]
pub enum VirtTextPos {
    Eol,
    Overlay,
    RightAlign,
    Inline,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, IntoEnumSC)]
pub enum HLMode {
    Replace,
    Combine,
    Blend,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HLText {
    pub text: String,
    pub highlight: String,
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
            out.push(tuple.into_lua(lua)?)?;
        }

        Ok(LuaValue::Table(out))
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TextType {
    String(String),
    Tuples(Vec<HLText>),
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

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LogLevel {
    Trace = 0,
    Debug = 1,
    Info = 2,
    Warn = 3,
    Error = 4,
    Off = 5,
}

#[derive(Clone, Copy, PartialEq, Eq, IntoEnumSC)]
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

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum OpenIn {
    Buffer,
    VSplit,
    HSplit,
    Tab,
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
#[derive(Debug, Default, Clone, IntoTable)]
pub struct ExtmarkOpts {
    // virt_lines : virtual lines to add next to this mark This
    // should be an array over lines, where each line in turn is
    // an array over `[text, highlight]` tuples. In general,
    // buffer and window options do not affect the display of the
    // text. In particular 'wrap' and 'linebreak' options do not
    // take effect, so the number of extra screen lines will
    // always match the size of the array. However the 'tabstop'
    // buffer option is still used for hard tabs. By default
    // lines are placed below the buffer line containing the
    // mark.
    // virt_lines_leftcol: Place extmarks in the leftmost column
    // of the window, bypassing sign and number columns.
    // ephemeral : for use with |nvim_set_decoration_provider()|
    // callbacks. The mark will only be used for the current
    // redraw cycle, and not be permantently stored in the
    // buffer.
    // right_gravity : boolean that indicates the direction the
    // extmark will be shifted in when new text is inserted (true
    // for right, false for left). Defaults to true.
    // end_right_gravity : boolean that indicates the direction
    // the extmark end position (if it exists) will be shifted in
    // when new text is inserted (true for right, false for
    // left). Defaults to false.
    // undo_restore : Restore the exact position of the mark if
    // text around the mark was deleted and then restored by
    // undo. Defaults to true.
    // invalidate : boolean that indicates whether to hide the
    // extmark if the entirety of its range is deleted. For
    // hidden marks, an "invalid" key is added to the "details"
    // array of |nvim_buf_get_extmarks()| and family. If
    // "undo_restore" is false, the extmark is deleted instead.
    // priority: a priority value for the highlight group, sign
    // attribute or virtual text. For virtual text, item with
    // highest priority is drawn last. For example treesitter
    // highlighting uses a value of 100.
    // strict: boolean that indicates extmark should not be
    // placed if the line or column value is past the end of the
    // buffer or end of the line respectively. Defaults to true.
    // sign_text: string of length 1-2 used to display in the
    // sign column.
    // sign_hl_group: name of the highlight group used to
    // highlight the sign column text.
    // number_hl_group: name of the highlight group used to
    // highlight the number column.
    // line_hl_group: name of the highlight group used to
    // highlight the whole line.
    // cursorline_hl_group: name of the highlight group used to
    // highlight the sign column text when the cursor is on the
    // same line as the mark and 'cursorline' is enabled.
    // conceal: string which should be either empty or a single
    // character. Enable concealing similar to |:syn-conceal|.
    // When a character is supplied it is used as |:syn-cchar|.
    // "hl_group" is used as highlight for the cchar if provided,
    // otherwise it defaults to |hl-Conceal|.
    // spell: boolean indicating that spell checking should be
    // performed within this extmark
    // ui_watched: boolean that indicates the mark should be
    // drawn by a UI. When set, the UI will receive win_extmark
    // events. Note: the mark is positioned by virt_text
    // attributes. Can be used together with virt_text.
    // url: A URL to associate with this extmark. In the TUI, the
    // OSC 8 control sequence is used to generate a clickable
    // hyperlink to this URL.
    // scoped: boolean that indicates that the extmark should
    // only be displayed in the namespace scope. (experimental)
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

#[derive(Debug, Default, IntoTable, Clone, Copy)]
pub struct KeymapOpts {
    pub silent: Option<bool>,
    pub buffer: Option<u32>,
}

impl KeymapOpts {
    pub fn new(buf_id: u32, silent: bool) -> Self {
        Self {
            buffer: Some(buf_id),
            silent: Some(silent),
        }
    }
}

#[derive(Debug, Clone, Copy, IntoTable, Default)]
pub struct BufferDeleteOpts {
    pub force: bool,
    pub unload: bool,
}

#[derive(Debug, Clone, Copy)]
pub struct WinCursor {
    row: u32,
    pub column: u32,
}

impl WinCursor {
    /// Create a cursor where the passed row argument starts from 0
    pub fn from_zero_indexed(row: u32, column: u32) -> Self {
        Self {
            row: row + 1,
            column,
        }
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
    fn from_lua(value: LuaValue<'a>, _lua: &'a Lua) -> LuaResult<Self> {
        match value {
            LuaValue::Table(table) => {
                let row: u32 = table.get(1)?;
                let column: u32 = table.get(2)?;

                Ok(Self { row, column })
            }
            _ => Err(LuaError::DeserializeError(
                "Supposed to be a table".to_string(),
            )),
        }
    }
}

impl<'a> IntoLua<'a> for WinCursor {
    fn into_lua(self, lua: &'a Lua) -> LuaResult<LuaValue<'a>> {
        let table = lua.create_table()?;
        table.set(1, self.row)?;
        table.set(2, self.column)?;

        Ok(LuaValue::Table(table))
    }
}

#[derive(Debug, FromTable, Clone)]
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

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Mode {
    Normal,
    Insert,
    Visual,
    Select,
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

//impl<'lua> FromLua<'lua> for Ui {
//fn from_lua(
//value: LuaValue<'lua>,
//lua: &'lua mlua::prelude::Lua,
//) -> mlua::prelude::LuaResult<Self> {
//lua.from_value(value)
//}
//}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct AutoCmd(u32);

impl AutoCmd {
    pub fn new(id: u32) -> Self {
        Self(id)
    }

    pub fn id(&self) -> u32 {
        self.0
    }
}

#[derive(Clone, Debug)]
pub enum AutoCmdGroup {
    String(String),
    Integer(u32),
}

#[derive(Debug, Clone, Copy, IntoEnum, PartialEq, Eq)]
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

#[derive(Debug)]
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

#[derive(Debug, Clone, FromTable)]
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

#[derive(Default, Clone)]
pub struct CmdOptsMagic {
    pub bar: bool,
    pub file: bool,
}

#[derive(Default, Clone)]
pub struct CmdOptsModsFilter<'a> {
    pub force: bool,
    pub pattern: &'a str,
}

#[derive(Clone)]
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
    pub tab: i32, // -1
    pub unsilent: bool,
    pub verbose: i32, // -1
    pub vertical: bool,
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
        // TODO make IntoTable complient to this structure
        out.set("args", self.args)?;
        out.set("bang", self.bang)?;

        Ok(LuaValue::Table(out))
    }
}

#[derive(Clone)]
pub struct CmdOpts<'a> {
    pub cmd: &'a str,
    pub args: &'a [&'a str],
    pub bang: bool,
}

impl<'a> CmdOpts<'a> {
    pub fn simple(cmd: &'a str) -> Self {
        Self {
            cmd,
            args: &[],
            bang: false,
        }
    }
}

pub struct FileTypeMatch {
    pub buf: Option<u32>,
    pub filename: Option<String>,
    pub contents: Option<Vec<String>>,
}

impl<'a> IntoLua<'a> for FileTypeMatch {
    fn into_lua(self, lua: &'a Lua) -> LuaResult<LuaValue<'a>> {
        let out = lua.create_table()?;

        if let Some(buf) = self.buf {
            out.set("buf", buf)?;
        }

        if let Some(filename) = self.filename {
            out.set("filename", filename)?;
        }

        if let Some(contents) = self.contents {
            out.set("contents", contents)?;
        }

        out.into_lua(lua)
    }
}
