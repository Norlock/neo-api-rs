use mlua::prelude::LuaResult;
use mlua::Lua;
use once_cell::sync::Lazy;
use std::cmp::Ordering;
use std::collections::HashSet;
use std::path::{Path, PathBuf};
use tokio::fs;
use tokio::io::{self};
use tokio::process::Command;
use tokio::sync::{Mutex, RwLock};
use tokio::time::Instant;

use crate::diffuser::{ChainResult, Diffuse, ExecuteTask};
use crate::web_devicons::icons_default::DevIcon;
use crate::{
    AutoCmdCbEvent, AutoCmdEvent, AutoCmdGroup, CmdOpts, Database, FileTypeMatch, HLOpts, Mode,
    NeoApi, NeoBuffer, NeoDebug, NeoPopup, NeoTheme, NeoWindow, OpenIn, PopupBorder, PopupRelative,
    PopupSize, PopupStyle, TextType, RTM,
};

const GRP_FUZZY_SELECT: &str = "NeoFuzzySelect";
const GRP_FUZZY_LETTER: &str = "NeoFuzzyLetter";
const AUCMD_GRP: &str = "neo-fuzzy";

#[derive(Clone, Debug, sqlx::FromRow)]
pub struct LineOut {
    pub text: String,
    pub icon: String,
    pub hl_group: String,
    pub id: u32,
}

struct FuzzyContainer {
    sorted_lines: RwLock<Vec<LineOut>>,
    fuzzy: RwLock<NeoFuzzy>,
    search_state: RwLock<SearchState>,
    preview: RwLock<Vec<String>>,
    db: Mutex<Database>,
}

pub trait FuzzyConfig: Send + Sync {
    fn cwd(&self, lua: &Lua) -> PathBuf;
    fn search_type(&self) -> FuzzySearch;
    fn on_enter(&self, lua: &Lua, open_in: OpenIn, item: PathBuf);
}

struct DummyConfig;

impl FuzzyConfig for DummyConfig {
    fn cwd(&self, _lua: &Lua) -> PathBuf {
        PathBuf::new()
    }

    fn on_enter(&self, _lua: &Lua, _open_in: OpenIn, _item: PathBuf) {}

    fn search_type(&self) -> FuzzySearch {
        FuzzySearch::Files
    }
}

impl std::fmt::Debug for dyn FuzzyConfig {
    fn fmt(&self, _f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        Ok(())
    }
}

#[derive(Debug)]
struct SearchState {
    last_search: String,
    file_path: String,
    update: bool,
}

static CONTAINER: Lazy<FuzzyContainer> = Lazy::new(|| FuzzyContainer {
    sorted_lines: RwLock::new(vec![]),
    fuzzy: RwLock::new(NeoFuzzy::default()),
    search_state: RwLock::new(SearchState {
        update: false,
        last_search: "".to_string(),
        file_path: "".to_string(),
    }),
    preview: RwLock::new(Vec::new()),
    db: Mutex::new({
        RTM.block_on(async move {
            let result = Database::init().await;

            if let Err(err) = result {
                NeoDebug::log(err.to_string()).await;
                panic!("");
            };

            result.unwrap()
        })
    }),
});

#[derive(Debug)]
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

impl Default for NeoFuzzy {
    fn default() -> Self {
        Self {
            pop_cmd: NeoPopup::default(),
            pop_out: NeoPopup::default(),
            pop_preview: NeoPopup::default(),
            cwd: PathBuf::new(),
            args: vec![],
            cmd: "".to_string(),
            selected_idx: 0,
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
}

impl FuzzySearch {
    /// Both Files + GitFiles
    pub fn is_file_based(&self) -> bool {
        match self {
            Self::Files | Self::GitFiles => true,
            _ => false,
        }
    }
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

        buf.set_keymap(
            lua,
            Mode::Insert,
            "<Enter>",
            lua.create_async_function(|lua, ()| open_item(lua, OpenIn::Buffer))?,
        )
    }

    pub async fn files_or_directories(lua: &Lua, config: Box<dyn FuzzyConfig>) -> LuaResult<()> {
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

        let out_preview_height = pop_cmd_row - 4;
        let out_preview_row = 2;

        let out_width = pop_cmd_width / 2;
        let out_col = pop_cmd_col;
        let preview_width = out_width - 2;
        let preview_col = pop_cmd_col + out_width + 2;

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
                title: Some(TextType::String(" Search query ".to_string())),
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
                border: crate::PopupBorder::Single,
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
                border: crate::PopupBorder::Single,
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

        let cmd = "fd".to_string();
        let cwd = config.cwd(lua);

        let args = match config.search_type() {
            FuzzySearch::Files | FuzzySearch::GitFiles => {
                vec!["--type".to_string(), "file".to_string()]
            }
            FuzzySearch::Directories => {
                vec!["--type".to_string(), "directory".to_string()]
            }
        };

        let fuzzy = NeoFuzzy {
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

        let cwd = fuzzy.cwd.clone();
        let cmd = fuzzy.cmd.clone();
        let args = fuzzy.args.clone();
        let search_type = fuzzy.config.search_type();

        Diffuse::queue([
            Box::new(ExecSearch {
                search_query: "".to_string(),
                cwd: cwd.clone(),
                cmd,
                args,
                search_type,
                failure_count: 0,
            }),
            Box::new(ExecPreview {
                cwd,
                failure_count: 0,
                selected_idx: 0,
            }),
        ])
        .await;

        *CONTAINER.fuzzy.write().await = fuzzy;
        CONTAINER.sorted_lines.write().await.clear();

        Diffuse::start().await;

        let interval = lua.create_function(interval_write_out)?;
        NeoApi::start_interval(lua, "fuzzy", 32, interval)?;

        Ok(())
    }

    pub fn fuzzy_grep(_cwd: &Path, _text: String) {
        //
    }

    fn add_preview_highlight(&self, lua: &Lua, preview: &[String]) -> LuaResult<()> {
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
            0,
            -1,
        )?;

        Ok(())
    }
}

pub async fn open_item(lua: &Lua, open_in: OpenIn) -> LuaResult<()> {
    let fuzzy = CONTAINER.fuzzy.read().await;
    let filtered_lines = CONTAINER.sorted_lines.read().await;

    let selected = fuzzy.cwd.join(&filtered_lines[fuzzy.selected_idx].text);

    fuzzy.pop_cmd.win.close(lua, false)?;
    fuzzy.config.on_enter(lua, open_in, selected);

    Ok(())
}

struct ExecSearch {
    cwd: PathBuf,
    cmd: String,
    args: Vec<String>,
    search_query: String,
    search_type: FuzzySearch,
    failure_count: usize,
}

impl ExecSearch {
    async fn insert_into_db(new_lines: Vec<LineOut>, instant: &Instant) {
        let mut new_lines: Vec<_> = new_lines
            .into_iter()
            .map(|line| (levenshtein("", &line.text), line))
            .collect();

        new_lines.sort_by_key(|kv| kv.0);

        let new_lines: Vec<_> = new_lines.into_iter().map(|line| line.1).collect();

        let before = instant.elapsed();

        let db = CONTAINER.db.lock().await;
        if let Err(err) = db.insert_all(&new_lines).await {
            NeoDebug::log(err).await;
        }

        let after = instant.elapsed();
        NeoDebug::log_duration(before, after, "insert collection").await;

        if let Ok(selection) = db.select("", instant).await {
            *CONTAINER.sorted_lines.write().await = selection;
        }
    }
}

impl ExecuteTask for ExecSearch {
    fn id(&self) -> &str {
        "search"
    }

    fn try_execute(self: Box<Self>) -> ChainResult {
        Box::pin(async move {
            let now = Instant::now();
            let first_search = CONTAINER.sorted_lines.read().await.is_empty();

            if first_search {
                let out = Command::new(&self.cmd)
                    .current_dir(&self.cwd)
                    .args(&self.args)
                    .output()
                    .await
                    .unwrap();

                if out.status.success() {
                    let out = String::from_utf8_lossy(&out.stdout);
                    let mut new_lines = Vec::new();

                    for (i, line) in out.lines().enumerate() {
                        if self.search_type == FuzzySearch::Directories {
                            new_lines.push(LineOut {
                                text: line.to_string(),
                                icon: "".to_string(),
                                hl_group: "Directory".to_string(),
                                id: i as u32,
                            });
                        } else {
                            let path = PathBuf::from(line);
                            let dev_icon = DevIcon::get_icon(&path);

                            new_lines.push(LineOut {
                                text: line.to_string(),
                                icon: dev_icon.icon.to_string(),
                                hl_group: dev_icon.highlight.to_string(),
                                id: i as u32,
                            });
                        }
                    }

                    NeoDebug::log(format!("lines: {}", new_lines.len())).await;
                    Self::insert_into_db(new_lines, &now).await;

                    let mut search_state = CONTAINER.search_state.write().await;
                    search_state.last_search = "".to_string();

                    let elapsed_ms = now.elapsed().as_millis();
                    NeoDebug::log(format!("elapsed search init: {}", elapsed_ms)).await;
                } else {
                    return Some(self as Box<dyn ExecuteTask>);
                }
            } else {
                let db = CONTAINER.db.lock().await;

                let query = if self.search_query.is_empty() {
                    "".to_string()
                } else {
                    let mut query = '%'.to_string();

                    for char in self.search_query.chars() {
                        query.push(char);
                        query.push('%');
                    }
                    query
                };

                if let Ok(selection) = db.select(&query, &now).await {
                    *CONTAINER.sorted_lines.write().await = selection;
                }

                let elapsed_ms = now.elapsed().as_millis();
                NeoDebug::log(format!("elapsed search: {}", elapsed_ms)).await;
            }

            None
        })
    }

    fn failure_count(&mut self) -> &mut usize {
        &mut self.failure_count
    }
}

async fn preview_file(path: &Path) -> io::Result<()> {
    let file_path;

    async fn handle_binary() {
        let mut preview = CONTAINER.preview.write().await;
        *preview = vec!["> File is a binary".to_string()];
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
                lines.push(line.to_string());
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
            let name = item.file_name().to_string_lossy().to_string();

            if file_type.is_dir() {
                items.push(format!("{name}/"));
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
        items.push("> Empty directory".to_string());
    }

    *CONTAINER.preview.write().await = items;

    Ok(())
}

struct ExecPreview {
    cwd: PathBuf,
    selected_idx: usize,
    failure_count: usize,
}

impl ExecuteTask for ExecPreview {
    fn id(&self) -> &str {
        "preview"
    }

    fn try_execute(self: Box<Self>) -> ChainResult {
        Box::pin(async move {
            let now = Instant::now();

            let path: PathBuf = {
                let filtered_lines = CONTAINER.sorted_lines.read().await;

                if filtered_lines.is_empty() {
                    CONTAINER.preview.write().await.clear();
                    CONTAINER.search_state.write().await.update = true;
                    return None;
                }

                self.cwd.join(&filtered_lines[self.selected_idx].text)
            };

            if path.is_dir() && preview_directory(&path).await.is_ok()
                || path.is_file() && preview_file(&path).await.is_ok()
            {
                CONTAINER.search_state.write().await.update = true;
                let elapsed_ms = now.elapsed().as_millis();
                NeoDebug::log(format!("elapsed preview: {}", elapsed_ms)).await;
                None
            } else {
                Some(self as Box<dyn ExecuteTask>)
            }
        })
    }

    fn failure_count(&mut self) -> &mut usize {
        &mut self.failure_count
    }
}

fn interval_write_out(lua: &Lua, _: ()) -> LuaResult<()> {
    fn execute(lua: &Lua) -> LuaResult<()> {
        let fuzzy = CONTAINER.fuzzy.try_read();
        let search_state = CONTAINER.search_state.try_write();
        let filtered_lines = CONTAINER.sorted_lines.try_read();
        let preview = CONTAINER.preview.try_write();

        if fuzzy.is_err() || search_state.is_err() || filtered_lines.is_err() || preview.is_err() {
            RTM.spawn(NeoDebug::log("denied"));
            return Ok(());
        }

        let sorted_lines = filtered_lines.unwrap().clone();
        let fuzzy = fuzzy.unwrap();
        let mut search_state = search_state.unwrap();
        let mut preview = preview.unwrap();

        let preview = std::mem::take(&mut *preview);

        if search_state.update {
            search_state.update = false;
            let file_path = search_state.file_path.to_string();

            drop(search_state);

            let mut icon_lines = Vec::new();
            let mut hl_groups = Vec::new();

            for line in sorted_lines.iter() {
                icon_lines.push(format!(" {} {}", line.icon, line.text));
                hl_groups.push(line.hl_group.as_str());
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
        }

        Ok(())
    }

    if let Err(err) = execute(lua) {
        NeoApi::notify(lua, &err)?;
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
        let cwd = fuzzy.cwd.clone();

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

        RTM.spawn(Diffuse::queue([Box::new(ExecPreview {
            cwd,
            failure_count: 0,
            selected_idx,
        })]));
    }

    Ok(())
}

fn close_fuzzy(lua: &Lua, _: ()) -> LuaResult<()> {
    NeoWindow::CURRENT.close(lua, true)
}

async fn aucmd_close_fuzzy(lua: &Lua, _ev: AutoCmdCbEvent) -> LuaResult<()> {
    let fuzzy = CONTAINER.fuzzy.read().await;

    Diffuse::stop().await;

    NeoApi::del_augroup_by_name(lua, AUCMD_GRP)?;
    NeoApi::stop_interval(lua, "fuzzy")?;
    NeoApi::set_insert_mode(lua, false)?;

    let db = CONTAINER.db.lock().await;
    RTM.spawn(async move {
        if let Err(err) = db.clean_up_tables().await {
            NeoDebug::log(err).await;
        }
    });

    fuzzy.pop_out.win.close(lua, false)?;
    fuzzy.pop_cmd.win.close(lua, false)?;
    fuzzy.pop_preview.win.close(lua, false)
}

async fn aucmd_text_changed(lua: &Lua, _ev: AutoCmdCbEvent) -> LuaResult<()> {
    let search_query = NeoApi::get_current_line(lua)?;
    let mut fuzzy = CONTAINER.fuzzy.write().await;

    fuzzy.selected_idx = 0;

    let cwd = fuzzy.cwd.clone();
    let cmd = fuzzy.cmd.clone();
    let args = fuzzy.args.clone();
    let search_type = fuzzy.config.search_type();
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

    let exec_search = Box::new(ExecSearch {
        search_query,
        cwd: cwd.clone(),
        cmd,
        args,
        search_type,
        failure_count: 0,
    });

    let exec_prev = Box::new(ExecPreview {
        cwd,
        failure_count: 0,
        selected_idx: 0,
    });

    RTM.spawn(Diffuse::queue([exec_search, exec_prev]));

    Ok(())
}

// TODO split on path
pub fn levenshtein(a: &str, b: &str) -> usize {
    let mut result = 0;

    /* Shortcut optimizations / degenerate cases. */
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

    /* Initialize the vector.
     *
     * This is why it’s fast, normally a matrix is used,
     * here we use a single vector. */
    let mut cache: Vec<usize> = (1..).take(length_a).collect();
    let mut distance_a;
    let mut distance_b;

    /* Loop. */
    for (index_b, code_b) in b.chars().enumerate() {
        result = index_b;
        distance_a = index_b;

        for (index_a, code_a) in a.chars().enumerate() {
            distance_b = if code_a == code_b {
                distance_a
            } else {
                distance_a + 1
            };

            distance_a = cache[index_a];

            result = if distance_a > result {
                if distance_b > result {
                    result + 1
                } else {
                    distance_b
                }
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
