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
    diffuser::TaskResult, web_devicons::DevIcon, ExecuteTask, FuzzySearch, LineOut, NeoApi,
    NeoDebug, StdpathType, CONTAINER,
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
    async fn execute(&self) -> TaskResult {
        if self.is_initial_search().await {
            self.insert_into_db().await
        } else {
            db_search(&self.search_query).await
        }
    }
}

impl ExecRecentDirectories {
    async fn insert_into_db(&self) -> TaskResult {
        let directories = fs::read(&self.recent_directories).await;

        if directories.is_err() {
            return TaskResult::default();
        }

        let directories = directories.unwrap();
        let directories_str = String::from_utf8_lossy(&directories);

        let mut new_lines = Vec::new();
        for line in directories_str.lines() {
            let root = format!("{}/", std::env!("HOME"));
            if let Some(line) = line.strip_prefix(&root) {
                new_lines.push(LineOut::directory(line.into()));
            }
        }

        let count = new_lines.len();
        if let Err(e) = CONTAINER.db.insert_all(&new_lines).await {
            NeoDebug::log(e).await;

            TaskResult::default()
        } else {
            *CONTAINER.search_lines.write().await = new_lines;

            TaskResult {
                db_count: Some(count),
                update: true,
                tabs: Some(vec![Box::new(" All directories "), Box::new(" Last used ")]),
                selected_tab: Some(0),
                selected_idx: Some(0),
            }
        }

        //let mut search_state = CONTAINER.search_state.write().await;
        //search_state.db_count = count;
        //search_state.update = true;
        //search_state.tabs = ;
        //search_state.selected_tab = 0;
    }
}

async fn db_search(search_query: &str) -> TaskResult {
    if let Ok(lines) = CONTAINER.db.search_lines(search_query).await {
        *CONTAINER.search_lines.write().await = lines;
    }

    TaskResult::default()
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

    fs::write(&recent_directories, out).await
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
    async fn execute(&self) -> TaskResult {
        if let Err(e) = remove_recent_directory(&self.recent_directories, self.index).await {
            NeoDebug::log(e).await;
        }

        TaskResult::default()
    }
}

pub struct ExecStandardSearch {
    pub cmd: &'static str,
    pub cwd: PathBuf,
    pub args: Vec<&'static str>,
    pub search_query: String,
    pub search_type: FuzzySearch,
}

pub struct ExecDirectorySearch {
    pub cmd: &'static str,
    pub cwd: PathBuf,
    pub args: Vec<&'static str>,
    pub search_query: String,
}

impl ExecStandardSearch {
    async fn insert_into_db(&self) {
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
                    new_lines.push(LineOut::directory(line.into()));
                } else {
                    let path = PathBuf::from(line);
                    let dev_icon = DevIcon::get_icon(&path);

                    new_lines.push(LineOut {
                        text: line.into(),
                        icon: dev_icon.icon.into(),
                        hl_group: dev_icon.highlight.into(),
                        git_root: None,
                    });
                }
            }

            CONTAINER.search_state.write().await.db_count = new_lines.len();

            match CONTAINER.db.insert_all(&new_lines).await {
                Ok(_) => {
                    if let Ok(new_lines) = CONTAINER.db.search_lines("").await {
                        *CONTAINER.search_lines.write().await = new_lines;
                    }
                }
                Err(err) => NeoDebug::log(err).await,
            }
        }
    }
}

#[async_trait::async_trait]
impl ExecuteTask for ExecStandardSearch {
    async fn execute(&self) -> TaskResult {
        let instant = Instant::now();

        if self.is_initial_search().await {
            self.insert_into_db().await;
        } else {
            db_search(&self.search_query).await;
        }

        let elapsed_ms = instant.elapsed().as_millis();
        NeoDebug::log(format!("elapsed search init: {}", elapsed_ms)).await;

        TaskResult::default()
    }
}

pub struct ClearResultsTask;

#[async_trait::async_trait]
impl ExecuteTask for ClearResultsTask {
    async fn execute(&self) -> TaskResult {
        CONTAINER.db.empty_lines().await;
        CONTAINER.search_lines.write().await.clear();

        //let mut search_state = CONTAINER.search_state.write().await;
        //search_state.db_count = 0;
        //search_state.selected_idx = 0;

        TaskResult {
            db_count: Some(0),
            selected_idx: Some(0),
            ..Default::default()
        }
    }
}

pub struct StoreRecentDirectory {
    pub recent_directories: PathBuf,
    pub dir_path: PathBuf,
}

#[async_trait::async_trait]
impl ExecuteTask for StoreRecentDirectory {
    // TODO store in DB??
    async fn execute(&self) -> TaskResult {
        if let Err(err) = self.store_directory().await {
            NeoDebug::log(err).await;
        }

        TaskResult::default()
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

    async fn store_directory(&self) -> io::Result<()> {
        let new_line = self.dir_path.to_string_lossy().to_string();

        let directories = fs::read(&self.recent_directories).await?;
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

        fs::write(&self.recent_directories, out).await
    }
}
