use mlua::{prelude::LuaResult, Lua};
use std::path::{Path, PathBuf};
use tokio::{
    fs::{self, OpenOptions},
    io::AsyncWriteExt,
};

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
            new_lines.push(LineOut::new_directory(line.into()));
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

    pub async fn store_directory(recent_directories: PathBuf, dir_path: PathBuf) {
        NeoDebug::log_dbg(&recent_directories).await;
        NeoDebug::log_dbg(&dir_path).await;

        let mut file = OpenOptions::new()
            .write(true)
            .create(true)
            .append(true)
            .open(recent_directories)
            .await
            .unwrap();

        let message = format!("{}\n", dir_path.to_string_lossy());

        let result = file.write_all(message.as_bytes()).await;

        if let Err(e) = result {
            RTM.spawn(NeoDebug::log(e));
        }
        //fs::write(path, contents)
    }
}
