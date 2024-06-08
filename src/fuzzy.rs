use mlua::prelude::LuaResult;
use mlua::Lua;
use once_cell::sync::Lazy;
use std::cmp::Ordering;
use std::collections::HashSet;
use std::path::{Path, PathBuf};
use tokio::fs;
use tokio::io::{self};
use tokio::process::Command;
use tokio::sync::RwLock;

use crate::diffuser::{ChainResult, Diffuse, ExecuteChain};
use crate::web_devicons::icons_default::DevIcon;
use crate::{
    AutoCmdCbEvent, AutoCmdEvent, AutoCmdGroup, CmdOpts, FileTypeMatch, HLOpts, Mode, NeoApi,
    NeoBuffer, NeoPopup, NeoTheme, NeoWindow, OpenIn, PopupBorder, PopupRelative, PopupSize,
    PopupStyle, TextType, RTM,
};

const GRP_FUZZY_SELECT: &str = "NeoFuzzySelect";
const GRP_FUZZY_LETTER: &str = "NeoFuzzyLetter";
const AUCMD_GRP: &str = "neo-fuzzy";

struct LineOut {
    text: String,
    icon: String,
    hl_group: String,
}

struct FuzzyContainer {
    lines_out: RwLock<Vec<LineOut>>,
    fuzzy: RwLock<NeoFuzzy>,
    search_state: RwLock<SearchState>,
    preview: RwLock<Vec<String>>,
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
        FuzzySearch::File
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
    update_out: bool,
    update_preview: bool,
}

static CONTAINER: Lazy<FuzzyContainer> = Lazy::new(|| FuzzyContainer {
    lines_out: RwLock::new(vec![]),
    fuzzy: RwLock::new(NeoFuzzy::default()),
    search_state: RwLock::new(SearchState {
        update_out: false,
        update_preview: false,
        last_search: "".to_string(),
        file_path: "".to_string(),
    }),
    preview: RwLock::new(Vec::new()),
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
    File,
    Directory,
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
            FuzzySearch::Directory => {
                vec!["--type".to_string(), "directory".to_string()]
            }
            FuzzySearch::File => {
                vec!["--type".to_string(), "file".to_string()]
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

        Diffuse::queue(Box::new(ExecSearch {
            search_query: "".to_string(),
            cwd,
            cmd,
            args,
            search_type,
            failure_count: 0,
        }))
        .await;

        Diffuse::start().await;

        *CONTAINER.fuzzy.write().await = fuzzy;

        let interval = lua.create_async_function(interval_write_out)?;

        NeoApi::start_interval(lua, "fuzzy", 32, interval)?;

        Ok(())
    }

    pub fn fuzzy_grep(_cwd: &Path, _text: String) {
        //
    }

    fn add_preview_highlight(&self, lua: &Lua, preview: &[String]) -> LuaResult<()> {
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

            if item_name.starts_with("> Empty directory")
                || item_name.starts_with("> File is a binary")
            {
                self.pop_preview
                    .buf
                    .add_highlight(lua, self.ns_id as i32, "Comment", i, 0, -1)?;
            }
        }

        Ok(())
    }

    fn add_out_highlight(&self, lua: &Lua) -> LuaResult<()> {
        self.pop_out
            .buf
            .clear_namespace(lua, self.ns_id as i32, 0, -1)?;

        self.pop_out.buf.add_highlight(
            lua,
            self.ns_id as i32,
            "NeoFuzzySelect",
            self.selected_idx,
            0,
            -1,
        )?;

        if let Ok(lines) = CONTAINER.lines_out.try_read() {
            let lines = if 300 <= lines.len() {
                &lines[..300]
            } else {
                &lines
            };

            for (i, line) in lines.iter().enumerate() {
                self.pop_out
                    .buf
                    .add_highlight(lua, self.ns_id as i32, &line.hl_group, i, 0, 2)?;
            }
        }

        Ok(())
    }
}

pub async fn open_item(lua: &Lua, open_in: OpenIn) -> LuaResult<()> {
    let fuzzy = CONTAINER.fuzzy.read().await;
    let cached_lines = CONTAINER.lines_out.read().await;
    let selected = fuzzy
        .cwd
        .join(cached_lines[fuzzy.selected_idx].text.as_str());
    fuzzy.pop_cmd.win.close(lua, false)?;

    fuzzy.config.on_enter(lua, open_in, selected);

    Ok(())
}

struct SortLines {
    lines_out: Vec<LineOut>,
    search_query: String,
    cwd: PathBuf,
    failure_count: usize,
}

impl ExecuteChain for SortLines {
    fn try_execute(self: Box<Self>) -> ChainResult {
        Box::pin(async move {
            let lines_out = CONTAINER.lines_out.try_write();
            let interval = CONTAINER.search_state.try_write();

            if lines_out.is_err() || interval.is_err() {
                return Some(self as Box<dyn ExecuteChain>);
            }

            let mut lines_out = lines_out.unwrap();
            let mut interval = interval.unwrap();

            *lines_out = self.lines_out;

            interval.last_search = self.search_query;
            interval.update_out = true;

            let preview = Box::new(ExecPreview {
                cwd: self.cwd,
                failure_count: 0,
            });

            preview.try_execute().await
        })
    }

    fn failure_count(&mut self) -> &mut usize {
        &mut self.failure_count
    }
}

struct ExecSearch {
    cwd: PathBuf,
    cmd: String,
    args: Vec<String>,
    search_query: String,
    search_type: FuzzySearch,
    failure_count: usize,
}

impl ExecuteChain for ExecSearch {
    fn try_execute(self: Box<Self>) -> ChainResult {
        Box::pin(async move {
            let sanitized = self.search_query.replace('/', "\\/").replace('.', "\\.");

            let mut regex = String::from(".*");

            for char in sanitized.chars() {
                if char.is_lowercase() {
                    regex.push_str(&format!("[{}{}]", char.to_uppercase(), char));
                } else {
                    regex.push(char);
                }

                if char != '\\' {
                    regex.push_str(".*");
                }
            }

            let search_specific_arg = if self.search_type == FuzzySearch::File {
                vec![regex.as_str()]
            } else {
                vec!["--full-path", &regex]
            };

            let out = Command::new(&self.cmd)
                .current_dir(&self.cwd)
                .args(&self.args)
                .args(search_specific_arg)
                .output()
                .await
                .unwrap();

            if out.status.success() {
                let lines = String::from_utf8_lossy(&out.stdout);
                let mut new_lines = Vec::new();

                for line in lines.lines() {
                    let score = levenshtein(&self.search_query, line);
                    new_lines.push((score, line.to_string()));
                }

                new_lines.sort_by_key(|k| k.0);

                let mut lines_out = Vec::new();

                for (_k, v) in new_lines.into_iter() {
                    if self.search_type == FuzzySearch::Directory {
                        lines_out.push(LineOut {
                            text: v,
                            icon: "".to_string(),
                            hl_group: "Directory".to_string(),
                        });
                    } else {
                        let path = PathBuf::from(&v);
                        if let Some(dev_icon) = DevIcon::get_icon(&path) {
                            lines_out.push(LineOut {
                                text: v,
                                icon: dev_icon.icon.to_string(),
                                hl_group: dev_icon.highlight.to_string(),
                            });
                        } else {
                            lines_out.push(LineOut {
                                text: v,
                                icon: "".to_string(),
                                hl_group: "".to_string(),
                            });
                        }
                    }
                }

                let sort_lines = Box::new(SortLines {
                    lines_out,
                    search_query: self.search_query.clone(),
                    failure_count: 0,
                    cwd: self.cwd,
                });

                sort_lines.try_execute().await
            } else {
                Some(self as Box<dyn ExecuteChain>)
            }
        })
    }

    fn failure_count(&mut self) -> &mut usize {
        &mut self.failure_count
    }
}

async fn preview_file(path: &Path) -> io::Result<()> {
    let file_path;

    if is_binary(path) {
        file_path = "text".to_string();

        let mut preview = CONTAINER.preview.write().await;
        *preview = vec!["> File is a binary".to_string()];
    } else {
        file_path = path.to_string_lossy().to_string();

        let mut lines = vec![];
        let file = fs::read_to_string(path).await?;

        for line in file.lines() {
            lines.push(line.to_string());
        }

        let mut preview = CONTAINER.preview.write().await;
        *preview = lines
    }

    let mut interval_state = CONTAINER.search_state.write().await;
    interval_state.file_path = file_path;
    interval_state.update_preview = true;

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
    CONTAINER.search_state.write().await.update_preview = true;

    Ok(())
}

struct ExecPreview {
    cwd: PathBuf,
    failure_count: usize,
}

impl ExecuteChain for ExecPreview {
    fn try_execute(self: Box<Self>) -> ChainResult {
        Box::pin(async move {
            let path: PathBuf = {
                let fuzzy = CONTAINER.fuzzy.try_read();
                let lines_out = CONTAINER.lines_out.try_read();

                if fuzzy.is_err() || lines_out.is_err() {
                    return Some(self as Box<dyn ExecuteChain>);
                }

                let lines_out = lines_out.unwrap();
                let selected_idx = fuzzy.unwrap().selected_idx;

                if lines_out.is_empty() {
                    CONTAINER.preview.write().await.clear();
                    CONTAINER.search_state.write().await.update_preview = true;
                    return None;
                }

                self.cwd.join(lines_out[selected_idx].text.as_str())
            };

            if path.is_dir() {
                if preview_directory(&path).await.is_ok() {
                    return None;
                }
            } else if path.is_file() {
                if preview_file(&path).await.is_ok() {
                    return None;
                }
            }

            Some(self)
        })
    }

    fn failure_count(&mut self) -> &mut usize {
        &mut self.failure_count
    }
}

async fn interval_write_out(lua: &Lua, _: ()) -> LuaResult<()> {
    let fuzzy = CONTAINER.fuzzy.try_read();
    let interval = CONTAINER.search_state.try_write();

    if fuzzy.is_err() || interval.is_err() {
        return Ok(());
    }

    let mut interval = interval.unwrap();
    let fuzzy = fuzzy.unwrap();

    let lines = CONTAINER.lines_out.try_read();

    if interval.update_out && lines.is_ok() {
        let lines = lines.unwrap();
        interval.update_out = false;

        let lines = if 300 <= lines.len() {
            &lines[..300]
        } else {
            &lines
        };

        if FuzzySearch::File == fuzzy.config.search_type() {
            let mut icon_lines = Vec::new();

            for line in lines {
                icon_lines.push(format!(" {} {}", line.icon, line.text));
            }

            fuzzy
                .pop_out
                .buf
                .set_lines(lua, 0, -1, false, &icon_lines)?;
        } else {
            let lines: Vec<_> = lines
                .iter()
                .map(|line| format!(" {} {}", line.icon, line.text))
                .collect();

            fuzzy.pop_out.buf.set_lines(lua, 0, -1, false, &lines)?;
        }

        fuzzy.add_out_highlight(lua)?;
    }

    let preview = CONTAINER.preview.try_read();
    
    if interval.update_preview && preview.is_ok() {
        interval.update_preview = false;
        let file_path = interval.file_path.to_string();

        fuzzy.add_out_highlight(lua)?;

        let buf = &fuzzy.pop_preview.buf;

        let preview = preview.unwrap();
        buf.set_lines(lua, 0, -1, false, &preview)?;
        fuzzy.add_preview_highlight(lua, &preview)?;

        if fuzzy.config.search_type() == FuzzySearch::File {
            let ft = NeoApi::filetype_match(
                lua,
                FileTypeMatch {
                    filename: Some(file_path),
                    contents: None,
                    buf: Some(buf.id()),
                },
            )?;

            buf.stop_treesitter(lua)?;

            if let Some(ft) = ft {
                let lang = buf.get_treesitter_lang(lua, &ft)?;

                if let Some(lang) = lang {
                    buf.start_treesitter(lua, &lang)?;
                }
            }
        }
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
        let sel_idx = fuzzy.selected_idx;
        let cwd = fuzzy.cwd.clone();

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

        drop(fuzzy);

        RTM.spawn(async move {
            let head = Box::new(ExecPreview {
                cwd,
                failure_count: 0,
            });

            Diffuse::queue(head).await;
        });
    }

    Ok(())
}

fn close_fuzzy(lua: &Lua, _: ()) -> LuaResult<()> {
    NeoWindow::CURRENT.close(lua, true)
}

async fn aucmd_close_fuzzy(lua: &Lua, _ev: AutoCmdCbEvent) -> LuaResult<()> {
    let fuzzy = CONTAINER.fuzzy.read().await;

    fuzzy.pop_out.win.close(lua, false)?;
    fuzzy.pop_cmd.win.close(lua, false)?;
    fuzzy.pop_preview.win.close(lua, false)?;

    Diffuse::stop().await;

    NeoApi::del_augroup_by_name(lua, AUCMD_GRP)?;
    NeoApi::stop_interval(lua, "fuzzy")?;
    NeoApi::set_insert_mode(lua, false)
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
        cwd,
        cmd,
        args,
        search_type,
        failure_count: 0,
    });

    RTM.spawn(Diffuse::queue(exec_search));

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
