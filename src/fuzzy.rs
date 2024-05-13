#![allow(unused)]
use mlua::Lua;
use mlua::{
    prelude::{LuaResult, LuaValue},
    IntoLua,
};
use once_cell::sync::Lazy;
use std::cmp::Ordering;
use std::io::BufReader;
use std::process::ExitStatus;
use std::{
    collections::HashMap,
    path::{Path, PathBuf},
};
use std::{process::Stdio, time::Instant};
use tokio::io::{self, AsyncWriteExt};
use tokio::process::*;
use tokio::runtime::Runtime;
use tokio::sync::RwLock;
use tokio::sync::{Mutex, RwLockReadGuard, TryLockError};
use tokio::{fs, join};

use crate::{
    callback, AutoCmdCbEvent, AutoCmdEvent, AutoCmdGroup, BufferDeleteOpts, CmdOpts, FastLock,
    HLOpts, Mode, NeoApi, NeoBuffer, NeoPopup, NeoTheme, NeoWindow, PopupBorder, PopupRelative,
    PopupSize, PopupSplit, PopupStyle, TextType,
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

#[derive(Debug)]
struct QueryMeta {
    last_search: String,
    update_results: bool,
}

unsafe impl Send for FuzzyContainer {}
unsafe impl Sync for FuzzyContainer {}

static CONTAINER: Lazy<FuzzyContainer> = Lazy::new(|| FuzzyContainer {
    all_lines: RwLock::new(String::new()),
    cached_lines: RwLock::new(vec![]),
    fuzzy: RwLock::new(None),
    rt: tokio::runtime::Runtime::new().unwrap(),
    query_meta: RwLock::new(QueryMeta {
        update_results: false,
        last_search: "".to_string(),
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
        self.pop_cmd.buf.set_keymap(
            lua,
            Mode::Insert,
            "<Up>",
            lua.create_async_function(|lua, _: ()| move_selection(lua, Move::Up))?,
        )?;

        self.pop_cmd.buf.set_keymap(
            lua,
            Mode::Insert,
            "<Down>",
            lua.create_async_function(|lua, _: ()| move_selection(lua, Move::Down))?,
        )?;

        self.pop_cmd.buf.set_keymap(
            lua,
            Mode::Insert,
            "<Esc>",
            lua.create_function(close_fuzzy)?,
        )
    }

    pub async fn files(lua: &Lua, cwd: PathBuf, search_type: FilesSearch) -> LuaResult<()> {
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
                pattern: vec![],
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
                pattern: vec![],
                once: true,
                desc: None,
            },
        )?;

        let cmd = "fd".to_string();

        let args = match search_type {
            FilesSearch::All => {
                vec![]
            }
            FilesSearch::DirOnly => {
                vec!["--type".to_string(), "directory".to_string()]
            }
            FilesSearch::FileOnly => {
                vec!["--type".to_string(), "file".to_string()]
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

    pub fn fuzzy_grep(cwd: &Path, text: String) {
        //
    }

    fn add_preview_highlight(&self, lua: &Lua, preview: &[String]) -> LuaResult<()> {
        self.pop_preview
            .buf
            .clear_namespace(lua, self.ns_id as i32, 0, -1)?;

        for (i, item_name) in preview.iter().enumerate() {
            if item_name.ends_with("/") {
                self.pop_preview.buf.add_highlight(
                    lua,
                    self.ns_id as i32,
                    "Directory",
                    i,
                    0,
                    -1,
                )?;
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

        Ok(())
    }
}

// TODO optimization to query from cached lines
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

        for (i, line) in lines.lines().enumerate() {
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
        let out = Command::new(cmd)
            .current_dir(cwd)
            .args(args)
            .output()
            .await?;

        if out.status.success() {
            let mut lines = CONTAINER.all_lines.write().await;
            *lines = String::from_utf8_lossy(&out.stdout).to_string();

            sort_lines(&lines, "").await;
        }
    } else {
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

        let mut rg_proc = Command::new("rg")
            .args(["--regexp", &regex])
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()?;

        let mut stdin = rg_proc.stdin.take().unwrap();

        let query_len = search_query.len();

        tokio::spawn(async move {
            let query_meta = CONTAINER.query_meta.read().await;

            if query_meta.last_search.len() < query_len {
                let lines = CONTAINER.cached_lines.read().await;
                stdin.write_all(lines.join("\n").as_bytes()).await.unwrap();
            } else {
                let lines = CONTAINER.all_lines.read().await;
                stdin.write_all(lines.as_bytes()).await.unwrap();
            }

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
                items.push(format!("{name}/"));
            } else {
                items.push(name);
            }
        }
    }

    items.sort_by(|a, b| {
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

    if query_meta.is_err() {
        return Ok(());
    }

    let query = query_meta.unwrap();

    if query.update_results {
        let fuzzy = CONTAINER.fuzzy.try_read();

        if fuzzy.is_err() {
            return Ok(());
        }

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
