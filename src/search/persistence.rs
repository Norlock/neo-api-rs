use mlua::{prelude::LuaResult, Lua};
use std::{
    collections::BTreeSet,
    io,
    path::{Path, PathBuf},
};
use tokio::{
    fs::{self},
    process::Command,
    time::Instant,
};

use crate::{
    web_devicons::icons_default::DevIcon, ExecuteTask, FuzzySearch, LineOut, NeoApi, NeoDebug,
    StdpathType, CONTAINER,
};

pub struct ExecRecentDirectories {
    pub recent_directories: PathBuf,
    pub search_query: String,
}

const RECENT_DIR_FILE: &str = "recent_directories.txt";
const HISTORY_COUNT: usize = 30;

// TODO use sqlite
#[async_trait::async_trait]
impl ExecuteTask for ExecRecentDirectories {
    async fn execute(&self) {
        if is_initial_search().await {
            insert_recent_directories_into_db(&self.recent_directories).await;
        } else {
            db_search(&self.search_query).await;
        }
    }
}

async fn insert_recent_directories_into_db(recent_directories: &Path) {
    let directories = fs::read(recent_directories).await;

    if directories.is_err() {
        return;
    }

    let directories = directories.unwrap();
    let directories_str = String::from_utf8_lossy(&directories);

    let mut new_lines = Vec::new();
    for line in directories_str.lines() {
        let root = format!("{}/", std::env!("HOME"));
        if let Some(line) = line.strip_prefix(&root) {
            new_lines.push(LineOut::new_directory(line.into()));
        }
    }

    let count = new_lines.len();
    let _ = CONTAINER.db.insert_all(&new_lines).await;
    *CONTAINER.search_lines.write().await = new_lines;

    let mut search_state = CONTAINER.search_state.write().await;
    search_state.db_count = count;
    search_state.update = true;
}

async fn db_search(search_query: &str) {
    if let Ok(lines) = CONTAINER.db.select(search_query).await {
        *CONTAINER.search_lines.write().await = lines;
        CONTAINER.search_state.write().await.update = true;
    }
}

async fn remove_recent_directory(recent_directories: &Path, index: usize) -> io::Result<()> {
    let directories = fs::read(&recent_directories).await?;
    let directories_str = String::from_utf8_lossy(&directories).to_string();

    let mut lines = Vec::new();

    for (i, line) in directories_str.lines().enumerate() {
        if i != index {
            lines.push(line.to_string());
        }
    }

    let out = lines.join("\n");

    fs::write(&recent_directories, out).await?;

    Ok(())
}

async fn is_initial_search() -> bool {
    CONTAINER.search_state.read().await.db_count == 0
}

impl ExecRecentDirectories {
    pub fn new(lua: &Lua, search_query: String) -> LuaResult<Self> {
        let stdpath = NeoApi::stdpath(lua, StdpathType::Data)?;

        Ok(Self {
            recent_directories: stdpath.join(RECENT_DIR_FILE),
            search_query,
        })
    }
}

pub struct RemoveRecentDirectory {
    pub recent_directories: PathBuf,
    pub index: usize,
}

impl RemoveRecentDirectory {
    pub fn new(lua: &Lua, index: usize) -> LuaResult<Self> {
        let stdpath = NeoApi::stdpath(lua, StdpathType::Data)?;

        Ok(Self {
            recent_directories: stdpath.join(RECENT_DIR_FILE),
            index,
        })
    }
}

#[async_trait::async_trait]
impl ExecuteTask for RemoveRecentDirectory {
    async fn execute(&self) {
        if remove_recent_directory(&self.recent_directories, self.index)
            .await
            .is_ok()
        {
            insert_recent_directories_into_db(&self.recent_directories).await;
        }
    }
}

pub struct ExecStandardSearch {
    pub cmd: &'static str,
    pub cwd: PathBuf,
    pub args: Vec<&'static str>,
    pub search_query: String,
    pub search_type: FuzzySearch,
}

impl ExecStandardSearch {
    async fn insert_fd_search_into_db(&self) {
        let out = Command::new(&self.cmd)
            .current_dir(&self.cwd)
            .args(&self.args)
            .output()
            .await
            .unwrap();

        if out.status.success() {
            let out = String::from_utf8_lossy(&out.stdout);
            let mut new_lines = Vec::new();

            for line in out.lines() {
                if self.search_type == FuzzySearch::Directories {
                    new_lines.push(LineOut::new_directory(line.into()));
                } else {
                    let path = PathBuf::from(line);
                    let dev_icon = DevIcon::get_icon(&path);

                    new_lines.push(LineOut {
                        text: line.into(),
                        icon: dev_icon.icon.into(),
                        hl_group: dev_icon.highlight.into(),
                    });
                }
            }

            CONTAINER.search_state.write().await.db_count = new_lines.len();

            if let Err(err) = CONTAINER.db.insert_all(&new_lines).await {
                NeoDebug::log(err).await;
            } else {
                db_search("").await;
            }
        }
    }
}

#[async_trait::async_trait]
impl ExecuteTask for ExecStandardSearch {
    async fn execute(&self) {
        let instant = Instant::now();

        if is_initial_search().await {
            self.insert_fd_search_into_db().await;
        } else {
            db_search(&self.search_query).await;
        }

        let elapsed_ms = instant.elapsed().as_millis();
        NeoDebug::log(format!("elapsed search init: {}", elapsed_ms)).await;
    }
}

pub struct ClearResultsTask;

#[async_trait::async_trait]
impl ExecuteTask for ClearResultsTask {
    async fn execute(&self) {
        CONTAINER.db.empty_lines().await;
    }
}

pub struct StoreRecentDirectory {
    pub recent_directories: PathBuf,
    pub dir_path: PathBuf,
}

#[async_trait::async_trait]
impl ExecuteTask for StoreRecentDirectory {
    // TODO store in DB
    async fn execute(&self) {
        if let Err(err) = store_directory(&self.recent_directories, &self.dir_path).await {
            NeoDebug::log(err).await;
        }
    }
}

impl StoreRecentDirectory {
    pub fn new(lua: &Lua, dir_path: PathBuf) -> LuaResult<Self> {
        let stdpath = NeoApi::stdpath(lua, StdpathType::Data)?;

        Ok(Self {
            recent_directories: stdpath.join(RECENT_DIR_FILE),
            dir_path,
        })
    }
}

async fn store_directory(recent_directories: &Path, dir_path: &Path) -> io::Result<()> {
    let new_line = dir_path.to_string_lossy().to_string();

    let directories = fs::read(recent_directories).await?;
    let directories_str = String::from_utf8_lossy(&directories).to_string();

    let mut lines = BTreeSet::from([new_line]);

    for line in directories_str.lines() {
        lines.insert(line.to_string());
    }

    let mut out = String::new();

    for (i, line) in lines.iter().enumerate() {
        if i == HISTORY_COUNT {
            break;
        }

        out.push_str(line);
        out.push_str("\n");
    }

    fs::write(recent_directories, out).await
}
