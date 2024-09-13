use mlua::prelude::{LuaError, LuaResult};
use mlua::Lua;
use std::path::{Path, PathBuf};
use std::sync::LazyLock;
use tokio::sync::{RwLock, RwLockReadGuard, RwLockWriteGuard};

use crate::search::{Diffuse, ExecuteTask};
use crate::web_devicons::DevIcon;
use crate::{
    AutoCmdCbEvent, AutoCmdEvent, AutoCmdGroup, ClearResultsTask, CmdOpts, Database, ExtmarkOpts,
    FileTypeMatch, HLOpts, HLText, InsertRecentDirectory, Mode, NeoApi, NeoBuffer, NeoDebug,
    NeoPopup, NeoTheme, NeoUtils, NeoWindow, OpenIn, PopupBorder, PopupRelative, PopupSize,
    PopupStyle, RemoveRecentDirectory, TextType, VirtTextPos, RTM,
};

use super::{ChangeTab, SearchState};

const GRP_FUZZY_SELECT: &str = "NeoFuzzySelect";
const GRP_FUZZY_LETTER: &str = "NeoFuzzyLetter";
const AUCMD_GRP: &str = "neo-fuzzy";
const TAB_BTN_SELECTED: &str = "TabButtonSelected";
const TAB_BTN: &str = "TabButton";

#[derive(Clone, Debug, Default, sqlx::FromRow)]
pub struct LineOut {
    pub text: Box<str>,
    pub icon: Box<str>,
    pub hl_group: Box<str>,
    pub git_root: Box<str>,
}

impl LineOut {
    pub fn directory(text: &str) -> Self {
        Self {
            text: text.into(),
            icon: "".into(),
            hl_group: "Directory".into(),
            git_root: "".into(),
        }
    }
}

pub struct FuzzyContainer {
    pub db: Database,
    pub fuzzy: RwLock<NeoFuzzy>,
    pub preview: RwLock<Vec<Box<str>>>,
    pub search_lines: RwLock<Vec<LineOut>>,
    pub search_state: RwLock<SearchState>,
}

pub trait FuzzyConfig: Send + Sync {
    fn cwd(&self) -> PathBuf;
    fn search_type(&self) -> FuzzySearch;
    fn search_task(&self, lua: &Lua, search_query: String, tab_idx: usize) -> Box<dyn ExecuteTask>;
    fn preview_task(&self, lua: &Lua, selected_idx: usize, tab_idx: usize) -> Box<dyn ExecuteTask>;
    fn on_enter(&self, lua: &Lua, open_in: OpenIn, item: PathBuf);
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
        panic!("not allowed");
    }

    fn preview_task(
        &self,
        _lua: &Lua,
        _selected_idx: usize,
        _tab_idx: usize,
    ) -> Box<dyn ExecuteTask> {
        panic!("not allowed");
    }
}

impl std::fmt::Debug for dyn FuzzyConfig {
    fn fmt(&self, _f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        Ok(())
    }
}

pub static CONTAINER: LazyLock<FuzzyContainer> = LazyLock::new(|| FuzzyContainer {
    search_lines: RwLock::new(vec![]),
    fuzzy: RwLock::new(NeoFuzzy::default()),
    search_state: RwLock::new(SearchState {
        update: false,
        db_count: 0,
        file_path: "".to_string(),
        tabs: vec![],
        selected_tab: 0,
        selected_idx: 0,
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
            lua.create_async_function(|lua, ()| SearchState::change_tab(lua, ChangeTab::Next))?,
        )?;

        buf.set_keymap(
            lua,
            Mode::Insert,
            "<S-Tab>",
            lua.create_async_function(|lua, ()| SearchState::change_tab(lua, ChangeTab::Previous))?,
        )
    }

    pub async fn open(lua: &Lua, config: Box<dyn FuzzyConfig>) -> LuaResult<()> {
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

        let fuzzy = NeoFuzzy {
            pop_cmd,
            pop_out,
            pop_preview,
            pop_tabs,
            ns_id,
            config,
        };

        fuzzy.add_keymaps(lua)?;

        Diffuse::queue([
            Box::new(ClearResultsTask),
            fuzzy.config.search_task(lua, "".to_string(), 0),
            fuzzy.config.preview_task(lua, 0, 0),
        ])
        .await;

        *CONTAINER.fuzzy.write().await = fuzzy;

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

    fn add_out_highlight(
        &self,
        lua: &Lua,
        hl_groups: Vec<&str>,
        selected_idx: usize,
    ) -> LuaResult<()> {
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
            selected_idx,
            3,
            -1,
        )?;

        Ok(())
    }
}

async fn open_item(lua: Lua, open_in: OpenIn) -> LuaResult<()> {
    let filtered_lines = CONTAINER.search_lines.read().await;

    if filtered_lines.is_empty() {
        return Ok(());
    }

    let fuzzy = CONTAINER.fuzzy.read().await;
    let fuzzy_c = &fuzzy.config;
    let search_state = CONTAINER.search_state.read().await;

    let selected = filtered_lines[search_state.selected_idx].text.as_ref();

    if fuzzy_c.search_type() == FuzzySearch::Directories && search_state.selected_tab == 0 {
        let home = NeoUtils::home_directory();
        let store_task = InsertRecentDirectory::new(home.join(selected));

        Diffuse::queue([Box::new(store_task)]).await;

        fuzzy.pop_cmd.win.close(&lua, false)?;
        fuzzy_c.on_enter(&lua, open_in, home.join(selected));
    } else if fuzzy_c.search_type() == FuzzySearch::Buffer {
        let search_state = CONTAINER.search_state.read().await;
        let root: PathBuf = search_state.tabs[search_state.selected_tab]
            .full()
            .as_ref()
            .into();

        fuzzy.pop_cmd.win.close(&lua, false)?;
        fuzzy_c.on_enter(&lua, open_in, root.join(selected));
    } else {
        fuzzy.pop_cmd.win.close(&lua, false)?;
        fuzzy_c.on_enter(&lua, open_in, selected.into());
    }

    Ok(())
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

            fuzzy.add_out_highlight(lua, hl_groups, search_state.selected_idx)?;

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

            let mut hl_texts: Vec<_> = vec![];

            for (i, tab) in search_state.tabs.iter().enumerate() {
                if i == search_state.selected_tab {
                    hl_texts.push(HLText::new(tab.name(), TAB_BTN_SELECTED.into()));
                } else {
                    hl_texts.push(HLText::new(tab.name(), TAB_BTN.into()));
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

async fn move_selection(lua: Lua, move_sel: Move) -> LuaResult<()> {
    let fuzzy = CONTAINER.fuzzy.read().await;
    let mut search_state = CONTAINER.search_state.write().await;

    let len = fuzzy.pop_out.buf.line_count(&lua)?;

    match move_sel {
        Move::Up => {
            if 0 < search_state.selected_idx {
                search_state.selected_idx -= 1;
            } else {
                search_state.selected_idx = len - 1;
            }
        }
        Move::Down => {
            if search_state.selected_idx + 1 < len {
                search_state.selected_idx += 1;
            } else {
                search_state.selected_idx = 0;
            }
        }
    }

    let selected_idx = search_state.selected_idx;

    fuzzy.pop_out.win.call(
        &lua,
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

    Diffuse::queue([fuzzy
        .config
        .preview_task(&lua, selected_idx, search_state.selected_tab)])
    .await;

    Ok(())
}

async fn delete_entry(lua: Lua, _: ()) -> LuaResult<()> {
    let fuzzy = CONTAINER.fuzzy.read().await;
    let fuzzy_c = &fuzzy.config;
    let search_state = CONTAINER.search_state.read().await;

    let st = fuzzy.config.search_type();

    let search_query = NeoApi::get_current_line(&lua)?;

    if st == FuzzySearch::Buffer {
        let search_lines = CONTAINER.search_lines.read().await;
        let selected = SearchState::get_selected(&fuzzy, &search_lines, search_state.selected_idx);

        NeoApi::cmd(
            &lua,
            CmdOpts {
                cmd: "bwipeout",
                args: &[selected.to_string_lossy().as_ref()],
                bang: false,
            },
        )?;

        Diffuse::queue([
            Box::new(ClearResultsTask),
            fuzzy_c.search_task(&lua, search_query, search_state.selected_tab),
            fuzzy_c.preview_task(&lua, search_state.selected_idx, search_state.selected_tab),
        ])
        .await;
    } else if st == FuzzySearch::Directories && search_state.selected_tab == 1 {
        let search_lines = CONTAINER.search_lines.read().await;
        let line = &search_lines[search_state.selected_idx];
        let remove_recent_dir = RemoveRecentDirectory::new(&line.text);

        Diffuse::queue([
            Box::new(remove_recent_dir),
            Box::new(ClearResultsTask),
            fuzzy_c.search_task(&lua, search_query, search_state.selected_tab),
            fuzzy_c.preview_task(&lua, search_state.selected_idx, search_state.selected_tab),
        ])
        .await;
    }

    Ok(())
}

fn close_fuzzy(lua: &Lua, _: ()) -> LuaResult<()> {
    NeoWindow::CURRENT.close(lua, true)
}

async fn aucmd_close_fuzzy(lua: Lua, _ev: AutoCmdCbEvent) -> LuaResult<()> {
    Diffuse::queue([Box::new(ClearResultsTask)]).await;

    NeoApi::del_augroup_by_name(&lua, AUCMD_GRP)?;
    NeoApi::stop_interval(&lua, "fuzzy")?;
    NeoApi::set_insert_mode(&lua, false)?;

    let fuzzy = CONTAINER.fuzzy.read().await;

    fuzzy.pop_out.win.close(&lua, false)?;
    fuzzy.pop_cmd.win.close(&lua, false)?;
    fuzzy.pop_preview.win.close(&lua, false)?;
    fuzzy.pop_tabs.win.close(&lua, false)?;

    Ok(())
}

async fn aucmd_text_changed(lua: Lua, _ev: AutoCmdCbEvent) -> LuaResult<()> {
    let fuzzy = CONTAINER.fuzzy.read().await;

    fuzzy.pop_out.win.call(
        &lua,
        lua.create_function(move |lua, _: ()| {
            NeoApi::cmd(
                lua,
                CmdOpts {
                    cmd: "normal",
                    bang: true,
                    args: &[&format!("1G")],
                },
            )
        })?,
    )?;

    let search_query = NeoApi::get_current_line(&lua)?;
    let selected_tab = CONTAINER.search_state.read().await.selected_tab;

    Diffuse::queue([
        fuzzy.config.search_task(&lua, search_query, selected_tab),
        fuzzy.config.preview_task(&lua, 0, selected_tab),
    ])
    .await;

    Ok(())
}
