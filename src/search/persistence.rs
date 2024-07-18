use mlua::{prelude::LuaResult, Lua};
use std::{collections::BTreeSet, io, path::PathBuf};
use tokio::fs::{self};

use crate::{ExecuteTask, LineOut, NeoApi, NeoDebug, StdpathType, CONTAINER, RTM};

pub struct ExecRecentDirectories {
    pub recent_directories: PathBuf,
}

const RECENT_DIR_FILE: &str = "recent_directories.txt";

#[async_trait::async_trait]
impl ExecuteTask for ExecRecentDirectories {
    async fn execute(&self) {
        let directories = fs::read(&self.recent_directories).await;

        if directories.is_err() {
            return;
        }

        let directories = directories.unwrap();
        let directories_str = String::from_utf8_lossy(&directories);

        let mut new_lines = Vec::new();

        for line in directories_str.lines().rev() {
            let root = format!("{}/", std::env!("HOME"));
            if let Some(line) = line.strip_prefix(&root) {
                new_lines.push(LineOut::new_directory(line.into()));
            }
        }

        RTM.spawn(NeoDebug::log_dbg(new_lines.clone()));

        *CONTAINER.search_lines.write().await = new_lines;
        CONTAINER.search_state.write().await.update = true;
    }
}

impl ExecRecentDirectories {
    pub fn new(lua: &Lua) -> LuaResult<Self> {
        let stdpath = NeoApi::stdpath(lua, StdpathType::Data)?;

        Ok(Self {
            recent_directories: stdpath.join(RECENT_DIR_FILE),
        })
    }

    pub async fn store_directory(recent_directories: PathBuf, dir_path: PathBuf) -> io::Result<()> {
        //let mut file = OpenOptions::new()
        //.write(true)
        //.create(true)
        //.append(true)
        //.open(recent_directories)
        //.await
        //.unwrap();

        let new_line = dir_path.to_string_lossy().to_string();

        let directories = fs::read(&recent_directories).await?;
        let directories_str = String::from_utf8_lossy(&directories).to_string();

        let mut lines = BTreeSet::from([new_line]);

        for line in directories_str.lines() {
            lines.insert(line.to_string());
        }

        let mut out = String::new();

        for (i, line) in lines.iter().enumerate() {
            if i == 50 {
                break;
            }

            out.push_str(line);
            out.push_str("\n");
        }

        let result = fs::write(recent_directories, out).await;

        if let Err(e) = result {
            RTM.spawn(NeoDebug::log(e));
        }

        Ok(())
    }
}
