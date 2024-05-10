#![allow(unused)]
use mlua::Lua;
use mlua::{
    prelude::{LuaResult, LuaValue},
    IntoLua,
};
use once_cell::sync::Lazy;
use std::process::Stdio;
use std::{
    collections::HashMap,
    path::{Path, PathBuf},
    process::Command,
    sync::Mutex,
};

use crate::{
    AutoCmdCbEvent, AutoCmdEvent, AutoCmdGroup, BufferDeleteOpts, CmdOpts, HLOpts, Mode, NeoApi,
    NeoBuffer, NeoPopup, NeoTheme, NeoWindow, PopupBorder, PopupSize, PopupSplit, PopupStyle,
    TextType,
};

const GRP_FUZZY_SELECT: &str = "NeoFuzzySelect";
const GRP_FUZZY_LETTER: &str = "NeoFuzzyLetter";

static CONTAINER: Lazy<Mutex<Option<NeoFuzzy>>> = Lazy::new(|| Mutex::new(None));

//  rg --no-heading --line-number . | rg -e NeoPop -e open
// rg --files | rg -e "e.*des.*ini.*"

// On match highlight with 'DiagnosticOk'

#[derive(Debug)]
pub struct NeoFuzzy {
    pub pop_cmd: NeoPopup,
    pub pop_out: NeoPopup,
    //pub pop_bat: NeoPopup,
    pub cwd: PathBuf,
    pub args: Vec<String>,
    pub cmd: String,

    pub selected_idx: usize,
    pub ns_id: u32,
    // Win command
    // Win choices
    // Win preview
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
    fn exec_search(&self, lua: &Lua, text: &str) -> LuaResult<()> {
        let text = text.replace('/', "\\/").replace('.', "\\.");

        let out;

        if text.is_empty() {
            out = Command::new(&self.cmd)
                .current_dir(&self.cwd)
                .args(&self.args)
                .output()?;
        } else {
            let mut regex = String::from(".*");

            for char in text.chars() {
                if char.is_lowercase() {
                    regex.push_str(&format!("[{}{}]", char.to_uppercase(), char));
                } else {
                    regex.push(char);
                }

                if char != '\\' {
                    regex.push_str(".*");
                }
            }

            NeoApi::notify(lua, &regex)?;

            let fd_cmd = Command::new(&self.cmd)
                .current_dir(&self.cwd)
                .args(&self.args)
                .stdout(Stdio::piped())
                .stderr(Stdio::piped())
                .spawn()?;

            let rg_cmd = Command::new("rg")
                .args(["--regexp", &regex])
                .stdin(Stdio::from(fd_cmd.stdout.unwrap()))
                .stdout(Stdio::piped())
                .stderr(Stdio::piped())
                .spawn()?;

            out = rg_cmd.wait_with_output()?;
        }

        if out.status.success() {
            let result = String::from_utf8_lossy(&out.stdout);

            let mut out = Vec::new();

            for (i, line) in result.lines().enumerate() {
                out.push(line.to_string());
                if i + 1 == 300 {
                    break;
                }
            }

            self.pop_out.buf.set_lines(lua, 0, -1, false, &out)?;
            self.add_highlight(lua)?;
        }

        Ok(())
    }

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

        let pop_cmd = NeoPopup::open(
            lua,
            NeoBuffer::create(lua, false, true)?,
            true,
            crate::WinOptions {
                width: Some(PopupSize::Percentage(1.0)),
                height: Some(PopupSize::Fixed(1)),
                row: Some(PopupSize::Fixed(1000)),
                border: PopupBorder::Single,
                title: Some(TextType::String("Search for directory".to_string())),
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
                callback: lua.create_function(aucmd_text_changed)?,
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

        let pop_out = NeoPopup::open(
            lua,
            NeoBuffer::create(lua, false, true)?,
            false,
            crate::WinOptions {
                width: Some(PopupSize::Percentage(0.5)),
                height: Some(PopupSize::Percentage(0.8)),
                win: Some(pop_cmd.win.id()),
                split: PopupSplit::Above,
                border: crate::PopupBorder::Single,
                focusable: Some(false),
                style: Some(PopupStyle::Minimal),
                ..Default::default()
            },
        )?;

        let mut container = CONTAINER.lock().unwrap();

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

        let fuzzy = NeoFuzzy {
            pop_cmd,
            pop_out,
            cwd,
            args,
            cmd,
            selected_idx: 0,
            ns_id,
        };

        fuzzy.add_keymaps(lua)?;
        fuzzy.exec_search(lua, "")?;

        *container = Some(fuzzy);

        // Preview buf

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

fn move_selection(lua: &Lua, move_sel: Move) -> LuaResult<()> {
    let mut fuzzy = CONTAINER.lock().unwrap();

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
    let mut container = CONTAINER.lock().unwrap();

    let buffer = NeoBuffer::new(ev.buf.unwrap());

    if let Some(fuzzy) = container.as_ref() {
        fuzzy.pop_out.win.close(lua, true)?;
        fuzzy.pop_cmd.win.close(lua, false)?;
    }

    *container = None;

    NeoApi::set_insert_mode(lua, false)
}

// TODO async search & sync loading
fn aucmd_text_changed(lua: &Lua, ev: AutoCmdCbEvent) -> LuaResult<()> {
    let buf_id = ev.buf.unwrap();
    NeoApi::notify(lua, &format!("even kijken {}", buf_id))?;

    let buf = NeoBuffer::new(buf_id);
    let text = NeoApi::get_current_line(lua)?;

    let fuzzy = CONTAINER.lock().unwrap();

    if let Some(fuzzy) = fuzzy.as_ref() {
        fuzzy.exec_search(lua, &text)?;
    }

    Ok(())
}
