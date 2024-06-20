use mlua::prelude::LuaResult;
use mlua::Lua;
use std::time::Duration;
use std::{env, fmt::Display};
use std::io::Write;
use tokio::{
    fs::{self, OpenOptions},
    io::{self, AsyncWriteExt},
};

use crate::{
    HLText, NeoBuffer, NeoPopup, PopupAlign, PopupBorder, PopupRelative, PopupSize, TextType,
    WinOptions,
};

pub struct NeoDebug;

impl NeoDebug {
    pub async fn log<AsStr: Display>(message: AsStr) {
        let mut dir = env::temp_dir();
        dir.push("neo-api-rs");

        let _ = fs::create_dir_all(&dir).await;

        let mut file = OpenOptions::new()
            .write(true)
            .create(true)
            .append(true)
            .open(dir.join("debug.log"))
            .await.unwrap();

        let mut bytes = vec![];
        let _ = writeln!(bytes, "{}", message);
        let _ = file.write_all(&bytes).await;
    }

    pub async fn log_duration(before: Duration, after: Duration, tag: &str) {
        let message = format!("{tag}: {}ms", after.as_millis() - before.as_millis()); 
        Self::log(message).await
    }

    pub async fn clear_logs() -> io::Result<()> {
        let log_file = env::temp_dir().join("neo-api-rs/debug.log");

        fs::write(log_file, b"").await
    }

    pub async fn display(lua: &Lua) -> LuaResult<()> {
        let buf = NeoBuffer::create(lua, false, true)?;

        // TODO readonly

        let mut log_file = env::temp_dir();
        log_file.push("neo-api-rs/debug.log");
        let log_file = fs::read_to_string(log_file).await?;

        let mut lines = vec![];

        for line in log_file.lines() {
            lines.push(line);
        }

        buf.set_lines(lua, 0, -1, false, &lines)?;

        let popup_win = NeoPopup::open_win(
            lua,
            &buf,
            true,
            WinOptions {
                relative: PopupRelative::Editor,
                width: Some(PopupSize::Percentage(0.8)),
                height: Some(PopupSize::Percentage(0.8)),
                col: Some(PopupSize::Percentage(0.1)),
                row: Some(PopupSize::Percentage(0.1)),
                style: None,
                border: PopupBorder::Rounded,
                title: Some(TextType::Tuples(vec![HLText::new("Debug logs", "Debug")])),
                title_pos: PopupAlign::Left,
                anchor: crate::Anchor::NorthWest,
                ..Default::default()
            },
        )?;

        buf.set_keymap(
            lua,
            crate::Mode::Normal,
            "<Esc>",
            lua.create_function(move |lua, ()| {
                popup_win.close(lua, false)?;
                Ok(())
            })?,
        )?;

        Ok(())
    }
}
