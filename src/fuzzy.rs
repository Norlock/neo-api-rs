#![allow(unused)]
use mlua::prelude::LuaResult;
use mlua::Lua;
use std::{path::Path, process::Command};

use crate::{NeoBuffer, NeoPopup, PopupSize, PopupSplit, PopupStyle};

//  rg --no-heading --line-number . | rg -e NeoPop -e open
// rg --files | rg -e "e.*des.*ini.*"

pub struct NeoFuzzy {
    pub command: Command,
    pub pop_cmd: NeoPopup,
    pub pop_out: NeoPopup,
    pub pop_bat: NeoPopup,
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
    pub async fn files(lua: &Lua, cwd: &Path, search_type: FilesSearch) -> LuaResult<()> {
        let pop_cmd = NeoPopup::open(
            lua,
            NeoBuffer::create(lua, false, true)?,
            true,
            crate::WinOptions {
                width: Some(PopupSize::Percentage(1.0)),
                height: Some(PopupSize::Fixed(1)),
                row: Some(PopupSize::Fixed(1000)),
                border: crate::PopupBorder::Single,
                ..Default::default()
            },
        )?;

        pop_cmd.buf.set_lines(lua, 0, 1, false, &["dummy data".to_string()]);

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
                ..Default::default()
            },
        )?;

        // Preview buf

        //let parts = text.split(" ");

        // TMP
        let text = "lua";

        let cmd = Command::new("fd")
            .current_dir(cwd)
            .args([text])
            .output()
            .expect("fd failed to run");

        if cmd.status.success() {
            let result = String::from_utf8_lossy(&cmd.stdout);

            let mut out = Vec::new();

            for line in result.lines() {
                out.push(line.to_string());
            }

            pop_out.buf.set_lines(lua, 0, -1, false, &out)?;
        }

        Ok(())
    }

    pub fn fuzzy_grep(cwd: &Path, text: String) {
        //
    }
}
