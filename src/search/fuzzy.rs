use mlua::prelude::{LuaError, LuaResult};
use mlua::Lua;
use std::cmp::Ordering;
use std::collections::HashSet;
use std::path::{Path, PathBuf};
use std::sync::LazyLock;
use tokio::fs;
use tokio::io::{self};
use tokio::sync::{RwLock, RwLockReadGuard, RwLockWriteGuard};
use tokio::time::Instant;

use crate::diffuser::{Diffuse, ExecuteTask};
use crate::web_devicons::icons_default::DevIcon;
use crate::{
    AutoCmdCbEvent, AutoCmdEvent, AutoCmdGroup, ClearResultsTask, CmdOpts, Database, DummyTask,
    ExtmarkOpts, FileTypeMatch, HLOpts, HLText, Mode, NeoApi, NeoBuffer, NeoDebug, NeoPopup,
    NeoTheme, NeoWindow, OpenIn, PopupBorder, PopupRelative, PopupSize, PopupStyle,
    RemoveRecentDirectory, StoreRecentDirectory, TextType, VirtTextPos, RTM,
};

use super::BufferSearch;

const GRP_FUZZY_SELECT: &str = "NeoFuzzySelect";
const GRP_FUZZY_LETTER: &str = "NeoFuzzyLetter";
const AUCMD_GRP: &str = "neo-fuzzy";
const TAB_BTN_SELECTED: &str = "TabButtonSelected";
const TAB_BTN: &str = "TabButton";

#[derive(Clone, Debug, sqlx::FromRow)]
pub struct LineOut {
    pub text: Box<str>,
    pub icon: Box<str>,
    pub hl_group: Box<str>,
}

impl LineOut {
    pub fn new_directory(text: Box<str>) -> Self {
        Self {
            text,
            icon: "".into(),
            hl_group: "Directory".into(),
        }
    }
}

pub struct FuzzyContainer {
    pub search_lines: RwLock<Vec<LineOut>>,
    pub fuzzy: RwLock<NeoFuzzy>,
    pub search_state: RwLock<SearchState>,
    pub preview: RwLock<Vec<Box<str>>>,
    pub db: Database,
}

pub trait FuzzyConfig: Send + Sync {
    fn cwd(&self) -> PathBuf;
    fn search_type(&self) -> FuzzySearch;
    fn search_task(&self, lua: &Lua, search_query: String, tab_idx: usize) -> Box<dyn ExecuteTask>;
    fn preview_task(&self, lua: &Lua, selected_idx: usize, tab_idx: usize) -> Box<dyn ExecuteTask>;
    fn on_enter(&self, lua: &Lua, open_in: OpenIn, item: PathBuf);
    fn tabs(&self) -> Vec<Box<str>>;
}

struct DummyConfig;

impl FuzzyConfig for DummyConfig {
    fn cwd(&self) -> PathBuf {
        PathBuf::new()
    }

    fn on_enter(&self, _lua: &Lua, _open_in: OpenIn, _item: PathBuf) {}

    fn search_type(&self) -> FuzzySearch {
        FuzzySearch::Files
    }

    fn search_task(
        &self,
        _lua: &Lua,
        _search_query: String,
        _tab_idx: usize,
    ) -> Box<dyn ExecuteTask> {
        Box::new(DummyTask)
    }

    fn preview_task(
        &self,
        _lua: &Lua,
        _selected_idx: usize,
        _tab_idx: usize,
    ) -> Box<dyn ExecuteTask> {
        Box::new(DummyTask)
    }

    fn tabs(&self) -> Vec<Box<str>> {
        vec![]
    }
}

impl std::fmt::Debug for dyn FuzzyConfig {
    fn fmt(&self, _f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        Ok(())
    }
}

#[derive(Debug)]
pub struct SearchState {
    pub file_path: String,
    pub db_count: usize,
    pub update: bool,
}

pub static CONTAINER: LazyLock<FuzzyContainer> = LazyLock::new(|| FuzzyContainer {
    search_lines: RwLock::new(vec![]),
    fuzzy: RwLock::new(NeoFuzzy::default()),
    search_state: RwLock::new(SearchState {
        update: false,
        db_count: 0,
        file_path: "".to_string(),
    }),
    preview: RwLock::new(Vec::new()),
    db: Database::new(),
});

#[derive(Debug)]
pub struct NeoFuzzy {
    pub pop_cmd: NeoPopup,
    pub pop_out: NeoPopup,
    pub pop_preview: NeoPopup,
    pub pop_tabs: NeoPopup,
    pub selected_idx: usize,
    pub selected_tab_idx: usize,
    pub tabs_count: usize,
    pub ns_id: u32,
    pub config: Box<dyn FuzzyConfig>,
}

impl Default for NeoFuzzy {
    fn default() -> Self {
        Self {
            pop_cmd: NeoPopup::default(),
            pop_out: NeoPopup::default(),
            pop_preview: NeoPopup::default(),
            pop_tabs: NeoPopup::default(),
            selected_idx: 0,
            tabs_count: 0,
            // TODO persist last use tabidx
            selected_tab_idx: 0,
            ns_id: 0,
            config: Box::new(DummyConfig),
        }
    }
}

#[derive(PartialEq, Eq, Clone, Copy)]
pub enum FuzzySearch {
    Files,
    Directories,
    GitFiles,
    Buffer,
}

impl FuzzySearch {
    /// Both Files + GitFiles
    pub fn is_file_based(&self) -> bool {
        match self {
            Self::Files | Self::GitFiles | Self::Buffer => true,
            _ => false,
        }
    }
}

enum Move {
    Up,
    Down,
}

impl NeoFuzzy {
    pub fn add_hl_groups(lua: &Lua) -> LuaResult<()> {
        NeoTheme::set_hl(
            lua,
            0,
            GRP_FUZZY_LETTER,
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

        NeoTheme::set_hl(
            lua,
            0,
            TAB_BTN_SELECTED,
            HLOpts {
                fg: Some("#ffffff".into()),
                //bg: Some("#1c96c5".into()),
                bg: Some("#151515".into()),
                ..Default::default()
            },
        )?;

        NeoTheme::set_hl(
            lua,
            0,
            TAB_BTN,
            HLOpts {
                fg: Some("#777777".into()),
                bg: Some("#151515".into()),
                ..Default::default()
            },
        )?;

        DevIcon::init(lua)?;

        Ok(())
    }

    pub fn add_keymaps(&self, lua: &Lua) -> LuaResult<()> {
        let buf = self.pop_cmd.buf;

        buf.set_keymap(
            lua,
            Mode::Insert,
            "<Up>",
            lua.create_async_function(|lua, ()| move_selection(lua, Move::Up))?,
        )?;

        buf.set_keymap(
            lua,
            Mode::Insert,
            "<Down>",
            lua.create_async_function(|lua, ()| move_selection(lua, Move::Down))?,
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
            "<C-t>",
            lua.create_async_function(|lua, ()| open_item(lua, OpenIn::Tab))?,
        )?;

        buf.set_keymap(
            lua,
            Mode::Insert,
            "<C-s>",
            lua.create_async_function(|lua, ()| open_item(lua, OpenIn::HSplit))?,
        )?;

        buf.set_keymap(
            lua,
            Mode::Insert,
            "<C-v>",
            lua.create_async_function(|lua, ()| open_item(lua, OpenIn::VSplit))?,
        )?;

        if self.config.search_type() == FuzzySearch::Directories
            || self.config.search_type() == FuzzySearch::Buffer
        {
            buf.set_keymap(
                lua,
                Mode::Insert,
                "<C-d>",
                lua.create_async_function(delete_entry)?,
            )?;
        }

        buf.set_keymap(
            lua,
            Mode::Insert,
            "<Enter>",
            lua.create_async_function(|lua, ()| open_item(lua, OpenIn::Buffer))?,
        )?;

        buf.set_keymap(
            lua,
            Mode::Insert,
            "<Tab>",
            lua.create_async_function(select_tab)?,
        )
    }

    pub async fn files_or_directories(lua: &Lua, config: Box<dyn FuzzyConfig>) -> LuaResult<()> {
        Self::add_hl_groups(lua)?;

        let ns_id = NeoTheme::create_namespace(lua, "NeoFuzzy")?;

        NeoTheme::set_hl_ns(lua, ns_id)?;

        let ui = &NeoApi::list_uis(lua)?[0];

        let pop_cmd_row = 2;
        let pop_cmd_col = 4;
        let pop_cmd_height = 1;
        let pop_cmd_width = if ui.width % 2 == 0 {
            ui.width - 8
        } else {
            ui.width - 9
        };

        let out_preview_height = ui.height - 10;
        let out_preview_row = pop_cmd_row + 3;

        let out_width = pop_cmd_width / 2;
        let out_col = pop_cmd_col;
        let preview_width = out_width - 2;
        let preview_col = pop_cmd_col + out_width + 2;

        let pop_cmd = NeoPopup::open(
            lua,
            NeoBuffer::create(lua, false, true)?,
            true,
            crate::WinOptions {
                width: Some(PopupSize::Fixed(out_width)),
                height: Some(PopupSize::Fixed(pop_cmd_height)),
                row: Some(PopupSize::Fixed(pop_cmd_row)),
                col: Some(PopupSize::Fixed(pop_cmd_col)),
                relative: PopupRelative::Editor,
                border: PopupBorder::Rounded,
                style: Some(PopupStyle::Minimal),
                title: Some(TextType::String(" Search ".to_string())),
                ..Default::default()
            },
        )?;

        let pop_tabs = NeoPopup::open(
            lua,
            NeoBuffer::create(lua, false, true)?,
            false,
            crate::WinOptions {
                width: Some(PopupSize::Fixed(preview_width)),
                height: Some(PopupSize::Fixed(pop_cmd_height)),
                row: Some(PopupSize::Fixed(pop_cmd_row)),
                col: Some(PopupSize::Fixed(preview_col)),
                relative: PopupRelative::Editor,
                border: PopupBorder::Rounded,
                style: Some(PopupStyle::Minimal),
                ..Default::default()
            },
        )?;

        let pop_out = NeoPopup::open(
            lua,
            NeoBuffer::create(lua, false, true)?,
            false,
            crate::WinOptions {
                width: Some(PopupSize::Fixed(out_width)),
                height: Some(PopupSize::Fixed(out_preview_height)),
                row: Some(PopupSize::Fixed(out_preview_row)),
                col: Some(PopupSize::Fixed(out_col)),
                relative: PopupRelative::Editor,
                border: crate::PopupBorder::Rounded,
                focusable: Some(false),
                style: Some(PopupStyle::Minimal),
                ..Default::default()
            },
        )?;

        pop_out.win.set_option_value(lua, "cursorline", true)?;

        let pop_preview = NeoPopup::open(
            lua,
            NeoBuffer::create(lua, false, true)?,
            false,
            crate::WinOptions {
                width: Some(PopupSize::Fixed(preview_width)),
                height: Some(PopupSize::Fixed(out_preview_height)),
                row: Some(PopupSize::Fixed(out_preview_row)),
                col: Some(PopupSize::Fixed(preview_col)),
                relative: PopupRelative::Editor,
                border: crate::PopupBorder::Rounded,
                focusable: Some(false),
                style: Some(PopupStyle::Minimal),
                noautocmd: true,
                ..Default::default()
            },
        )?;

        pop_cmd.buf.set_current(lua)?;
        NeoApi::set_insert_mode(lua, true)?;

        let group = NeoApi::create_augroup(lua, AUCMD_GRP, false)?;

        let callback = lua.create_async_function(aucmd_text_changed)?;

        NeoApi::create_autocmd(
            lua,
            &[AutoCmdEvent::TextChangedI],
            crate::AutoCmdOpts {
                callback,
                buffer: Some(pop_cmd.buf.id()),
                group: Some(AutoCmdGroup::Integer(group)),
                pattern: vec![],
                once: false,
                desc: None,
            },
        )?;

        NeoApi::create_autocmd(
            lua,
            &[AutoCmdEvent::BufLeave],
            crate::AutoCmdOpts {
                callback: lua.create_async_function(aucmd_close_fuzzy)?,
                buffer: Some(pop_cmd.buf.id()),
                group: Some(AutoCmdGroup::Integer(group)),
                pattern: vec![],
                once: true,
                desc: None,
            },
        )?;

        let tabs_count = config.tabs().len();

        let fuzzy = NeoFuzzy {
            pop_cmd,
            pop_out,
            pop_preview,
            pop_tabs,
            selected_idx: 0,
            selected_tab_idx: 0,
            tabs_count,
            ns_id,
            config,
        };

        fuzzy.add_keymaps(lua)?;

        Diffuse::queue(vec![
            fuzzy
                .config
                .search_task(lua, "".to_string(), fuzzy.selected_tab_idx),
            fuzzy.config.preview_task(lua, 0, fuzzy.selected_tab_idx),
        ])
        .await;

        *CONTAINER.fuzzy.write().await = fuzzy;
        CONTAINER.search_lines.write().await.clear();

        Diffuse::start().await;

        let interval = lua.create_function(interval_write_out)?;
        NeoApi::start_interval(lua, "fuzzy", 32, interval)?;

        Ok(())
    }

    pub fn fuzzy_grep(_cwd: &Path, _text: String) {
        //
    }

    fn add_preview_highlight(&self, lua: &Lua, preview: &[Box<str>]) -> LuaResult<()> {
        if self.config.search_type() == FuzzySearch::Directories {
            self.pop_preview
                .buf
                .clear_namespace(lua, self.ns_id as i32, 0, -1)?;

            for (i, item_name) in preview.iter().enumerate() {
                if item_name.ends_with('/') {
                    self.pop_preview.buf.add_highlight(
                        lua,
                        self.ns_id as i32,
                        "Directory",
                        i,
                        0,
                        -1,
                    )?;
                }

                if item_name.starts_with("> Empty directory") {
                    self.pop_preview.buf.add_highlight(
                        lua,
                        self.ns_id as i32,
                        "Comment",
                        i,
                        0,
                        -1,
                    )?;
                }
            }
        } else if preview.len() == 1 && preview[0].starts_with("> File is a binary") {
            self.pop_preview
                .buf
                .add_highlight(lua, self.ns_id as i32, "Comment", 0, 0, -1)?;
        }

        Ok(())
    }

    fn add_out_highlight(&self, lua: &Lua, hl_groups: Vec<&str>) -> LuaResult<()> {
        self.pop_out
            .buf
            .clear_namespace(lua, self.ns_id as i32, 0, -1)?;

        for (i, hl_group) in hl_groups.into_iter().enumerate() {
            self.pop_out
                .buf
                .add_highlight(lua, self.ns_id as i32, hl_group, i, 0, 2)?;
        }

        self.pop_out.buf.add_highlight(
            lua,
            self.ns_id as i32,
            "NeoFuzzySelect",
            self.selected_idx,
            3,
            -1,
        )?;

        Ok(())
    }
}

async fn select_tab(lua: &Lua, _: ()) -> LuaResult<()> {
    let mut fuzzy = CONTAINER.fuzzy.write().await;

    if fuzzy.selected_tab_idx + 1 < fuzzy.tabs_count {
        fuzzy.selected_tab_idx += 1;
    } else {
        fuzzy.selected_tab_idx = 0;
    }

    let search_query = NeoApi::get_current_line(lua)?;

    Diffuse::queue(vec![
        Box::new(ClearResultsTask),
        fuzzy
            .config
            .search_task(lua, search_query, fuzzy.selected_tab_idx),
        fuzzy
            .config
            .preview_task(lua, fuzzy.selected_idx, fuzzy.selected_tab_idx),
    ])
    .await;

    Ok(())
}

async fn exec_default_search(lua: &Lua, fuzzy: &NeoFuzzy) -> LuaResult<()> {
    let search_query = NeoApi::get_current_line(lua)?;

    Diffuse::queue(vec![
        fuzzy
            .config
            .search_task(lua, search_query, fuzzy.selected_tab_idx),
        fuzzy
            .config
            .preview_task(lua, fuzzy.selected_idx, fuzzy.selected_tab_idx),
    ])
    .await;

    //RTM.spawn(NeoDebug::log(format!("Mem usage: {:?}", lua.used_memory())));

    Ok(())
}

async fn open_item(lua: &Lua, open_in: OpenIn) -> LuaResult<()> {
    let fuzzy = CONTAINER.fuzzy.read().await;
    let filtered_lines = CONTAINER.search_lines.read().await;

    if filtered_lines.is_empty() {
        return Ok(());
    }

    let selected = fuzzy
        .config
        .cwd()
        .join(filtered_lines[fuzzy.selected_idx].text.as_ref());

    if fuzzy.config.search_type() == FuzzySearch::Directories && fuzzy.selected_tab_idx != 1 {
        let store_task = StoreRecentDirectory::new(lua, selected.clone())?;

        Diffuse::queue(vec![Box::new(store_task)]).await;
    }

    fuzzy.pop_cmd.win.close(lua, false)?;
    fuzzy.config.on_enter(lua, open_in, selected);

    Ok(())
}

async fn preview_file(path: &Path) -> io::Result<()> {
    let file_path;

    async fn handle_binary() {
        let mut preview = CONTAINER.preview.write().await;
        *preview = vec!["> File is a binary".into()];
    }

    if is_binary(path) {
        file_path = "text".to_string();
        handle_binary().await;
    } else {
        // TODO check for ZIP and preview
        if let Ok(file) = fs::read_to_string(path).await {
            file_path = path.to_string_lossy().to_string();

            let mut lines = vec![];

            for line in file.lines() {
                lines.push(line.into());
            }

            *CONTAINER.preview.write().await = lines;
        } else {
            file_path = "text".to_string();
            handle_binary().await;
        }
    }

    let mut search_state = CONTAINER.search_state.write().await;
    search_state.file_path = file_path;

    Ok(())
}

async fn preview_directory(path: &Path) -> io::Result<()> {
    let mut items = Vec::new();
    let mut dir = fs::read_dir(path).await?;

    while let Some(item) = dir.next_entry().await? {
        if let Ok(file_type) = item.file_type().await {
            let name = item.file_name().to_string_lossy().into();

            if file_type.is_dir() {
                items.push(format!("{name}/").into_boxed_str());
            } else {
                items.push(name);
            }
        }
    }

    items.sort_by(|a, b| {
        if a.ends_with('/') == b.ends_with('/') {
            a.cmp(b)
        } else if a.ends_with('/') {
            Ordering::Less
        } else {
            Ordering::Greater
        }
    });

    if items.is_empty() {
        items.push("> Empty directory".into());
    }

    *CONTAINER.preview.write().await = items;

    Ok(())
}

pub struct ExecPreview {
    pub cwd: PathBuf,
    pub selected_idx: usize,
}

#[async_trait::async_trait]
impl ExecuteTask for ExecPreview {
    async fn execute(&self) {
        let now = Instant::now();

        let path: PathBuf = {
            let filtered_lines = CONTAINER.search_lines.read().await;

            if filtered_lines.is_empty() {
                CONTAINER.preview.write().await.clear();
                CONTAINER.search_state.write().await.update = true;
                return;
            }

            self.cwd
                .join(filtered_lines[self.selected_idx].text.as_ref())
        };

        if path.is_dir() && preview_directory(&path).await.is_ok()
            || path.is_file() && preview_file(&path).await.is_ok()
        {
            CONTAINER.search_state.write().await.update = true;
            let elapsed_ms = now.elapsed().as_millis();
            NeoDebug::log(format!("elapsed preview: {}", elapsed_ms)).await;
        }
    }
}

trait NeoTryLock<'a, T: ?Sized> {
    fn interval_read(&'a self) -> LuaResult<RwLockReadGuard<'a, T>>;
    fn interval_write(&'a self) -> LuaResult<RwLockWriteGuard<'a, T>>;
}

impl<'a, T: ?Sized> NeoTryLock<'a, T> for RwLock<T> {
    fn interval_read(&'a self) -> LuaResult<RwLockReadGuard<'a, T>> {
        self.try_read().map_err(LuaError::external)
    }

    fn interval_write(&'a self) -> LuaResult<RwLockWriteGuard<'a, T>> {
        self.try_write().map_err(LuaError::external)
    }
}

fn interval_write_out(lua: &Lua, _: ()) -> LuaResult<()> {
    fn execute(lua: &Lua) -> LuaResult<()> {
        let fuzzy = CONTAINER.fuzzy.interval_read()?;
        let search_lines = CONTAINER.search_lines.interval_read()?;
        let mut search_state = CONTAINER.search_state.interval_write()?;
        let mut preview = CONTAINER.preview.interval_write()?;

        let preview = std::mem::take(&mut *preview);

        if search_state.update {
            search_state.update = false;
            let file_path = search_state.file_path.to_string();
            let info_text = format!(" ({}/{}) ", search_lines.len(), search_state.db_count);

            drop(search_state);

            let mut icon_lines = Vec::new();
            let mut hl_groups = Vec::new();

            for line in search_lines.iter() {
                icon_lines.push(format!(" {} {}", line.icon, line.text));
                hl_groups.push(line.hl_group.as_ref());
            }

            fuzzy
                .pop_out
                .buf
                .set_lines(lua, 0, -1, false, &icon_lines)?;

            fuzzy.add_out_highlight(lua, hl_groups)?;

            let buf = &fuzzy.pop_preview.buf;

            buf.set_lines(lua, 0, -1, false, &preview)?;

            if fuzzy.config.search_type().is_file_based() {
                let ft = NeoApi::filetype_match(
                    lua,
                    FileTypeMatch {
                        filename: Some(file_path),
                        contents: None,
                        buf: Some(buf.id()),
                    },
                )?;

                if let Some(ft) = ft {
                    let lang = buf.get_treesitter_lang(lua, &ft)?;

                    if let Some(lang) = lang {
                        buf.start_treesitter(lua, &lang)?;
                    }
                } else {
                    buf.stop_treesitter(lua)?;
                }
            }

            fuzzy.add_preview_highlight(lua, &preview)?;

            let buf = &fuzzy.pop_cmd.buf;

            let opts = ExtmarkOpts {
                id: Some(333),
                virt_text: Some(vec![HLText::new(info_text, "Comment".to_string())]),
                virt_text_pos: Some(VirtTextPos::RightAlign),
                ..Default::default()
            };

            buf.set_extmarks(lua, fuzzy.ns_id, 0, 0, opts)?;

            let buf = &fuzzy.pop_tabs.buf;

            // TODO fetch tabs somewhere
            let mut hl_texts: Vec<_> = vec![];

            for (i, tab) in fuzzy.config.tabs().iter().enumerate() {
                if i == fuzzy.selected_tab_idx {
                    hl_texts.push(HLText::new(tab.as_ref(), TAB_BTN_SELECTED));
                } else {
                    hl_texts.push(HLText::new(tab.as_ref(), TAB_BTN));
                }

                hl_texts.push(HLText::new(" ", ""));
            }

            let opts = ExtmarkOpts {
                id: Some(333),
                virt_text: Some(hl_texts),
                ..Default::default()
            };

            buf.set_extmarks(lua, fuzzy.ns_id, 0, 0, opts)?;
        }

        Ok(())
    }

    if let Err(err) = execute(lua) {
        RTM.spawn(NeoDebug::log(err));
    }

    Ok(())
}

async fn move_selection(lua: &Lua, move_sel: Move) -> LuaResult<()> {
    let mut fuzzy = CONTAINER.fuzzy.write().await;

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
        let selected_idx = fuzzy.selected_idx;

        fuzzy.pop_out.win.call(
            lua,
            lua.create_function(move |lua, _: ()| {
                NeoApi::cmd(
                    lua,
                    CmdOpts {
                        cmd: "normal",
                        bang: true,
                        args: &[&format!("{}G", selected_idx + 1)],
                    },
                )
            })?,
        )?;

        Diffuse::queue(vec![fuzzy.config.preview_task(
            lua,
            selected_idx,
            fuzzy.selected_tab_idx,
        )])
        .await;
    }

    Ok(())
}

async fn delete_entry(lua: &Lua, _: ()) -> LuaResult<()> {
    let fuzzy = CONTAINER.fuzzy.read().await;

    let st = fuzzy.config.search_type();

    let search_query = NeoApi::get_current_line(lua)?;

    let tasks: Option<Vec<Box<dyn ExecuteTask>>> = if st == FuzzySearch::Buffer {
        let search_lines = CONTAINER.search_lines.read().await;
        let selected = fuzzy
            .config
            .cwd()
            .join(search_lines[fuzzy.selected_idx].text.as_ref());

        drop(search_lines);

        NeoApi::cmd(
            lua,
            CmdOpts {
                cmd: "bwipeout",
                args: &[selected.to_string_lossy().as_ref()],
                bang: false,
            },
        )?;

        Some(vec![])
    } else if st == FuzzySearch::Directories && fuzzy.selected_tab_idx == 1 {
        let remove_recent_dir = RemoveRecentDirectory::new(lua, fuzzy.selected_idx)?;

        Some(vec![Box::new(remove_recent_dir)])
    } else {
        None
    };

    if let Some(mut tasks) = tasks {
        tasks.push(Box::new(ClearResultsTask));
        tasks.push(
            fuzzy
                .config
                .search_task(lua, search_query, fuzzy.selected_tab_idx),
        );

        tasks.push(
            fuzzy
                .config
                .preview_task(lua, fuzzy.selected_idx, fuzzy.selected_tab_idx),
        );

        Diffuse::queue(tasks).await;
    }

    Ok(())
}

fn close_fuzzy(lua: &Lua, _: ()) -> LuaResult<()> {
    NeoWindow::CURRENT.close(lua, true)
}

async fn aucmd_close_fuzzy(lua: &Lua, _ev: AutoCmdCbEvent) -> LuaResult<()> {
    Diffuse::queue(vec![Box::new(ClearResultsTask)]).await;
    Diffuse::stop().await;

    NeoApi::del_augroup_by_name(lua, AUCMD_GRP)?;
    NeoApi::stop_interval(lua, "fuzzy")?;
    NeoApi::set_insert_mode(lua, false)?;

    let fuzzy = CONTAINER.fuzzy.read().await;

    fuzzy.pop_out.win.close(lua, false)?;
    fuzzy.pop_cmd.win.close(lua, false)?;
    fuzzy.pop_preview.win.close(lua, false)?;
    fuzzy.pop_tabs.win.close(lua, false)?;


    Ok(())
}

async fn aucmd_text_changed(lua: &Lua, _ev: AutoCmdCbEvent) -> LuaResult<()> {
    let mut fuzzy = CONTAINER.fuzzy.write().await;

    fuzzy.selected_idx = 0;

    let sel_idx = fuzzy.selected_idx;

    fuzzy.pop_out.win.call(
        lua,
        lua.create_function(move |lua, _: ()| {
            NeoApi::cmd(
                lua,
                CmdOpts {
                    cmd: "normal",
                    bang: true,
                    args: &[&format!("{}G", sel_idx + 1)],
                },
            )
        })?,
    )?;

    exec_default_search(lua, &fuzzy).await?;

    Ok(())
}

fn is_binary(file: &Path) -> bool {
    let binaries: HashSet<&str> = [
        "bin", "so", "mkv", "mp4", "blend", "jpg", "png", "jpeg", "webp",
    ]
    .into();

    if let Some(ext) = file.extension() {
        binaries.contains(ext.to_str().unwrap())
    } else {
        false
    }
}
