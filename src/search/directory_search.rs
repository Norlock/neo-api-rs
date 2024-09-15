use std::path::{Path, PathBuf};
use tokio::{process::Command, time::Instant};

use crate::{search::TaskResult, ExecuteTask, FuzzyTab, LineOut, NeoDebug, CONTAINER};

use super::PreviewTask;

fn tabs() -> Option<Vec<Box<dyn FuzzyTab>>> {
    Some(vec![
        Box::new(" All directories "),
        Box::new(" Recent directories "),
    ])
}

pub struct ExecRecentDirectories {
    pub search_query: String,
}

impl ExecRecentDirectories {
    pub fn new(search_query: String) -> Self {
        Self { search_query }
    }

    async fn search_recent_directories(&self) -> TaskResult {
        match CONTAINER
            .db
            .search_recent_directories(&self.search_query)
            .await
        {
            Ok(new_lines) => {
                let db_count = new_lines.len();
                *CONTAINER.search_lines.write().await = new_lines;

                TaskResult {
                    db_count: Some(db_count),
                    selected_idx: Some(0),
                    selected_tab: Some(1),
                    tabs: tabs(),
                    ..Default::default()
                }
            }
            Err(err) => {
                NeoDebug::log(err).await;
                TaskResult::default()
            }
        }
    }
}

// TODO use sqlite
#[async_trait::async_trait]
impl ExecuteTask for ExecRecentDirectories {
    async fn execute(&self) -> TaskResult {
        let instant = Instant::now();

        let result = self.search_recent_directories().await;

        let elapsed_ms = instant.elapsed().as_millis();
        NeoDebug::log(format!("Elapsed recent search: {}", elapsed_ms)).await;

        result
    }
}

async fn db_search(search_query: &str, cwd: &Path) -> TaskResult {
    if let Ok(new_lines) = CONTAINER.db.search_lines(search_query).await {
        if new_lines.is_empty() {
            TaskResult {
                new_lines: Some(vec![]),
                selected_idx: Some(0),
                update: true,
                ..Default::default()
            }
        } else {
            let path_prefix = cwd.to_path_buf();
            let path_suffix = new_lines[0].text.clone();

            let preview = PreviewTask::new(path_prefix, path_suffix).execute().await;

            TaskResult {
                new_lines: Some(vec![]),
                selected_idx: Some(0),
                update: true,
                preview_lines: preview.preview_lines,
                file_path: preview.file_path,
                ..Default::default()
            }
        }
    } else {
        TaskResult::default()
    }
}

pub struct RemoveRecentDirectory {
    path: Box<str>,
}

impl RemoveRecentDirectory {
    pub fn new(path: &str) -> Self {
        Self { path: path.into() }
    }
}

#[async_trait::async_trait]
impl ExecuteTask for RemoveRecentDirectory {
    async fn execute(&self) -> TaskResult {
        CONTAINER.db.delete_recent_directory(&self.path).await;

        TaskResult::default()
    }
}

pub struct ExecDirectorySearch {
    pub cmd: &'static str,
    pub cwd: PathBuf,
    pub args: Vec<&'static str>,
    pub search_query: String,
}

impl ExecDirectorySearch {
    async fn insert_into_db(&self) -> TaskResult {
        let out = Command::new(self.cmd)
            .current_dir(&self.cwd)
            .args(&self.args)
            .output()
            .await
            .unwrap();

        if out.status.success() {
            let out = String::from_utf8_lossy(&out.stdout);
            let mut new_lines = Vec::new();

            for line in out.lines() {
                new_lines.push(LineOut::directory(line));
            }

            let db_count = new_lines.len();

            match CONTAINER.db.insert_all(&new_lines).await {
                Ok(_) => match CONTAINER.db.search_lines("").await {
                    Ok(new_lines) => {
                        let path_prefix = self.cwd.clone();
                        let path_suffix = new_lines[0].text.clone();

                        let preview = PreviewTask::new(path_prefix, path_suffix).execute().await;

                        return TaskResult {
                            db_count: Some(db_count),
                            selected_idx: Some(0),
                            selected_tab: Some(0),
                            new_lines: Some(new_lines),
                            line_prefix: Some(self.cwd.clone()),
                            update: true,
                            tabs: tabs(),
                            preview_lines: preview.preview_lines,
                            file_path: preview.file_path,
                            ..Default::default()
                        };
                    }
                    Err(e) => NeoDebug::log(e).await,
                },
                Err(e) => NeoDebug::log(e).await,
            }
        }

        TaskResult::default()
    }
}

#[async_trait::async_trait]
impl ExecuteTask for ExecDirectorySearch {
    async fn execute(&self) -> TaskResult {
        let instant = Instant::now();

        let result = if self.all_lines_is_empty().await {
            NeoDebug::log("is initial search").await;
            self.insert_into_db().await
        } else {
            db_search(&self.search_query, &self.cwd).await
        };

        let elapsed_ms = instant.elapsed().as_millis();
        NeoDebug::log(format!("Elapsed directory search: {}", elapsed_ms)).await;

        result
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
            selected_tab: Some(0),
            ..Default::default()
        }
    }
}

pub struct InsertRecentDirectory(PathBuf);

impl InsertRecentDirectory {
    pub fn new(directory: PathBuf) -> Self {
        Self(directory)
    }
}

#[async_trait::async_trait]
impl ExecuteTask for InsertRecentDirectory {
    async fn execute(&self) -> TaskResult {
        CONTAINER
            .db
            .insert_recent_directory(self.0.to_string_lossy())
            .await;

        TaskResult::default()
    }
}
