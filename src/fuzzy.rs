#![allow(unused)]
use mlua::Lua;
use mlua::{
    prelude::{LuaResult, LuaValue},
    IntoLua,
};
use once_cell::sync::Lazy;
use std::{
    collections::HashMap,
    path::{Path, PathBuf},
};
use std::{process::Stdio, time::Instant};
use tokio::io;
use tokio::join;
use tokio::process::*;
use tokio::runtime::Runtime;
use tokio::sync::Mutex;
use tokio::sync::RwLock;

use crate::{
    AutoCmdCbEvent, AutoCmdEvent, AutoCmdGroup, BufferDeleteOpts, CmdOpts, FastLock, HLOpts, Mode,
    NeoApi, NeoBuffer, NeoPopup, NeoTheme, NeoWindow, PopupBorder, PopupRelative, PopupSize,
    PopupSplit, PopupStyle, TextType,
};

const GRP_FUZZY_SELECT: &str = "NeoFuzzySelect";
const GRP_FUZZY_LETTER: &str = "NeoFuzzyLetter";

struct FuzzyContainer {
    lines: RwLock<Vec<String>>,
    fuzzy: Mutex<Option<NeoFuzzy>>,
    rt: Mutex<Runtime>,
}

unsafe impl Send for FuzzyContainer {}
unsafe impl Sync for FuzzyContainer {}

static CONTAINER: Lazy<FuzzyContainer> = Lazy::new(|| FuzzyContainer {
    lines: RwLock::new(vec![]),
    fuzzy: Mutex::new(None),
    rt: Mutex::new(tokio::runtime::Runtime::new().unwrap()),
});

#[derive(Debug)]
pub struct NeoFuzzy {
    pub pop_cmd: NeoPopup,
    pub pop_out: NeoPopup,
    pub pop_bat: NeoPopup,
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

async fn exec_search(
    cwd: PathBuf,
    cmd: String,
    args: Vec<String>,
    search_query: &str,
) -> tokio::io::Result<()> {
    let sanitized = search_query.replace('/', "\\/").replace('.', "\\.");

    let out;

    if sanitized.is_empty() {
        out = Command::new(cmd)
            .current_dir(cwd)
            .args(args)
            .output()
            .await?;
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

        let mut fd_cmd = Command::new(cmd)
            .current_dir(cwd)
            .args(args)
            .stdout(Stdio::piped())
            .spawn()?;

        let rg_stdin: Stdio = fd_cmd
            .stdout
            .take()
            .unwrap()
            .try_into()
            .expect("failed to convert to Stdio");

        let rg_cmd = Command::new("rg")
            .args(["--regexp", &regex])
            .stdin(rg_stdin)
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()?;

        let (fd_result, tr_output) = join!(fd_cmd.wait(), rg_cmd.wait_with_output());

        if fd_result.is_err() {
            return Err(io::Error::new(io::ErrorKind::Interrupted, "Cmd fd failed"));
        }

        out = tr_output.expect("failed to await tr");
    }

    if out.status.success() {
        let result = String::from_utf8_lossy(&out.stdout);

        let mut out = Vec::new();

        for (i, line) in result.lines().enumerate() {
            let score = levenshtein(&search_query, line);
            out.push((score, line.to_string()));
        }

        out.sort_by_key(|k| k.0);

        let mut lines = CONTAINER.lines.write().await;
        *lines = out.into_iter().map(|k| k.1).collect();
    }

    Ok(())
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
            lua.create_function(|lua, _: ()| move_selection(lua, Move::Up))?,
        )?;

        self.pop_cmd.buf.set_keymap(
            lua,
            Mode::Insert,
            "<Down>",
            lua.create_function(|lua, _: ()| move_selection(lua, Move::Down))?,
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

        let pop_bat = NeoPopup::open(
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

        NeoApi::create_autocmd(
            lua,
            &[AutoCmdEvent::TextChangedI],
            crate::AutoCmdOpts {
                callback: lua.create_async_function(aucmd_text_changed)?,
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
                callback: lua.create_function(close_fuzzy_aucmd)?,
                buffer: Some(pop_cmd.buf.id()),
                group: Some(AutoCmdGroup::Integer(group)),
                pattern: vec![],
                once: true,
                desc: None,
            },
        )?;

        let mut container = CONTAINER.fuzzy.lock().await;

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
            pop_bat,
            cwd,
            args,
            cmd,
            selected_idx: 0,
            ns_id,
        };

        fuzzy.add_keymaps(lua)?;

        let rt = CONTAINER.rt.lock().await;
        let cwd = fuzzy.cwd.clone();
        let cmd = fuzzy.cmd.clone();
        let args = fuzzy.args.clone();

        rt.spawn(async move {
            exec_search(cwd, cmd, args, "").await;
        });

        *container = Some(fuzzy);

        let interval = lua.create_async_function(read_to_out)?;

        NeoApi::start_interval(lua, "fuzzy", 100, interval)?;

        Ok(())
    }

    pub fn fuzzy_grep(cwd: &Path, text: String) {
        //
    }

    fn add_highlight(&self, lua: &Lua) -> LuaResult<()> {
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

async fn read_to_out(lua: &Lua, _: ()) -> LuaResult<()> {
    let fuzzy = CONTAINER.fuzzy.lock().await;
    let lines = CONTAINER.lines.read().await;

    if let Some(fuzzy) = fuzzy.as_ref() {
        if 300 <= lines.len() {
            let _ = fuzzy
                .pop_out
                .buf
                .set_lines(lua, 0, -1, false, &lines[..300]);
        } else {
            let _ = fuzzy.pop_out.buf.set_lines(lua, 0, -1, false, &lines);
        }
        let _ = fuzzy.add_highlight(lua);
    }

    Ok(())
}

fn move_selection(lua: &Lua, move_sel: Move) -> LuaResult<()> {
    let mut fuzzy = CONTAINER.fuzzy.blocking_lock();

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
        }

        fuzzy.add_highlight(lua)?;
    }

    Ok(())
}

fn close_fuzzy(lua: &Lua, _: ()) -> LuaResult<()> {
    NeoWindow::CURRENT.close(lua, true)
}

fn close_fuzzy_aucmd(lua: &Lua, ev: AutoCmdCbEvent) -> LuaResult<()> {
    let mut container = CONTAINER.fuzzy.blocking_lock();

    let buffer = NeoBuffer::new(ev.buf.unwrap());

    if let Some(fuzzy) = container.as_ref() {
        fuzzy.pop_out.win.close(lua, false)?;
        fuzzy.pop_cmd.win.close(lua, false)?;
        fuzzy.pop_bat.win.close(lua, false)?;
    }

    NeoApi::stop_interval(lua, "fuzzy")?;

    *container = None;

    NeoApi::set_insert_mode(lua, false)
}

async fn aucmd_text_changed(lua: &Lua, ev: AutoCmdCbEvent) -> LuaResult<()> {
    let buf_id = ev.buf.unwrap();

    let buf = NeoBuffer::new(buf_id);
    let search_query = NeoApi::get_current_line(lua)?;

    let mut fuzzy = CONTAINER.fuzzy.lock().await;

    if let Some(fuzzy) = fuzzy.as_mut() {
        fuzzy.selected_idx = 0;

        let rt = CONTAINER.rt.lock().await;
        let cwd = fuzzy.cwd.clone();
        let cmd = fuzzy.cmd.clone();
        let args = fuzzy.args.clone();

        rt.spawn(async move {
            exec_search(cwd, cmd, args, &search_query).await;
        });
    }

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
