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
    process::Command,
    sync::Mutex,
};

use crate::{
    AutoCmdCbEvent, AutoCmdEvent, AutoCmdGroup, BufferDeleteOpts, Mode, NeoApi, NeoBuffer,
    NeoPopup, NeoWindow, PopupSize, PopupSplit, PopupStyle, TextType,
};

static CONTAINER: Lazy<Mutex<HashMap<NeoBuffer, NeoFuzzy>>> =
    Lazy::new(|| Mutex::new(HashMap::new()));

//  rg --no-heading --line-number . | rg -e NeoPop -e open
// rg --files | rg -e "e.*des.*ini.*"

#[derive(Debug)]
pub struct NeoFuzzy {
    pub pop_cmd: NeoPopup,
    pub pop_out: NeoPopup,
    //pub pop_bat: NeoPopup,
    pub cwd: PathBuf,
    pub args: Vec<String>,
    pub cmd: String,
    // Win command
    // Win choices
    // Win preview
}

pub enum FilesSearch {
    FileOnly,
    DirOnly,
    All,
}

impl NeoFuzzy {
    pub async fn files(lua: &Lua, cwd: PathBuf, search_type: FilesSearch) -> LuaResult<()> {
        let pop_cmd = NeoPopup::open(
            lua,
            NeoBuffer::create(lua, false, true)?,
            true,
            crate::WinOptions {
                width: Some(PopupSize::Percentage(1.0)),
                height: Some(PopupSize::Fixed(1)),
                row: Some(PopupSize::Fixed(1000)),
                border: crate::PopupBorder::Single,
                title: Some(TextType::String("Search for directory".to_string())),
                ..Default::default()
            },
        )?;

        pop_cmd.buf.set_current(lua)?;
        pop_cmd.buf.set_keymap(
            lua,
            Mode::Insert,
            "<Esc>",
            lua.create_function(close_fuzzy)?,
        )?;

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

        let mut fuzziers = CONTAINER.lock().unwrap();

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

        fuzziers.insert(
            pop_cmd.buf,
            NeoFuzzy {
                pop_cmd,
                pop_out,
                cwd,
                args,
                cmd,
            },
        );

        let fuzzy = fuzziers.get(&pop_cmd.buf).unwrap();
        exec_search(lua, fuzzy, "")?;

        // Preview buf

        Ok(())
    }

    pub fn fuzzy_grep(cwd: &Path, text: String) {
        //
    }
}

fn close_fuzzy(lua: &Lua, _: ()) -> LuaResult<()> {
    NeoWindow::CURRENT.close(lua, true)
}

fn close_fuzzy_aucmd(lua: &Lua, ev: AutoCmdCbEvent) -> LuaResult<()> {
    let mut container = CONTAINER.lock().unwrap();

    let buffer = NeoBuffer::new(ev.buf.unwrap());
    let fuzzy = container.remove(&buffer).unwrap();

    fuzzy.pop_out.win.close(lua, true)?;
    fuzzy.pop_cmd.win.close(lua, false)?;

    NeoApi::set_insert_mode(lua, false)
}

// TODO async search & sync loading
fn exec_search(lua: &Lua, fuzzy: &NeoFuzzy, text: &str) -> LuaResult<()> {
    let regex = text.replace('/', "\\/").replace('.', "\\.");
    let mut out = String::new();

    for char in regex.chars() {
        out.push(char);

        if char.is_alphanumeric() {
            out.push_str(".*");
        }
    }

    NeoApi::notify(lua, &out)?;

    let cmd = Command::new(&fuzzy.cmd)
        .current_dir(&fuzzy.cwd)
        .args(&fuzzy.args)
        .arg(out)
        .output()
        .expect("Command failed to run");

    if cmd.status.success() {
        let result = String::from_utf8_lossy(&cmd.stdout);

        let mut out = Vec::new();

        for line in result.lines() {
            out.push(line.to_string());
        }

        fuzzy.pop_out.buf.set_lines(lua, 0, -1, false, &out)?;
    }

    Ok(())
}

fn aucmd_text_changed(lua: &Lua, ev: AutoCmdCbEvent) -> LuaResult<()> {
    let buf_id = ev.buf.unwrap();
    NeoApi::notify(lua, &format!("even kijken {}", buf_id))?;

    let buf = NeoBuffer::new(buf_id);
    let text = NeoApi::get_current_line(lua)?;

    let fuzziers = CONTAINER.lock().unwrap();
    let fuzzy = fuzziers.get(&buf).unwrap();

    exec_search(lua, fuzzy, &text)
}
