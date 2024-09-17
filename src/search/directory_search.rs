use std::path::PathBuf;
use tokio::{process::Command, time::Instant};

use crate::{search::TaskResult, ExecuteTask, FuzzyTab, LineOut, NeoDebug, CONTAINER};

use super::Preview;

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
                let preview_lines = if new_lines.is_empty() {
                    Some(vec![])
                } else {
                    Some(Preview::get_lines(new_lines[0].clone()).await)
                };

                TaskResult {
                    db_count: Some(db_count),
                    selected_idx: Some(0),
                    selected_tab: Some(1),
                    search_lines: Some(new_lines),
                    preview_lines,
                    tabs: tabs(),
                    update: true,
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

async fn db_search(search_query: &str) -> TaskResult {
    if let Ok(new_lines) = CONTAINER.db.search_lines(search_query).await {
        let preview_lines = if new_lines.is_empty() {
            Some(vec![])
        } else {
            Some(Preview::get_lines(new_lines[0].clone()).await)
        };

        TaskResult {
            search_lines: Some(new_lines),
            selected_idx: Some(0),
            update: true,
            preview_lines,
            ..Default::default()
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
            let path_prefix = self.cwd.to_string_lossy();

            for path_suffix in out.lines() {
                new_lines.push(LineOut::directory(&path_prefix, path_suffix));
            }

            let db_count = new_lines.len();

            match CONTAINER.db.insert_all(&new_lines).await {
                Ok(_) => match CONTAINER.db.search_lines("").await {
                    Ok(new_lines) => {
                        let preview_lines = if new_lines.is_empty() {
                            Some(vec![])
                        } else {
                            Some(Preview::get_lines(new_lines[0].clone()).await)
                        };

                        return TaskResult {
                            db_count: Some(db_count),
                            selected_idx: Some(0),
                            selected_tab: Some(0),
                            search_lines: Some(new_lines),
                            preview_lines,
                            update: true,
                            tabs: tabs(),
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
            self.insert_into_db().await
        } else {
            db_search(&self.search_query).await
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

        TaskResult {
            db_count: Some(0),
            selected_idx: Some(0),
            search_lines: Some(vec![]),
            ..Default::default()
        }
    }
}

pub struct InsertRecentDirectory(LineOut);

impl InsertRecentDirectory {
    pub fn new(line_out: LineOut) -> Self {
        Self(line_out)
    }
}

#[async_trait::async_trait]
impl ExecuteTask for InsertRecentDirectory {
    async fn execute(&self) -> TaskResult {
        CONTAINER
            .db
            .insert_recent_directory(&self.0.path_prefix, &self.0.path_suffix)
            .await;

        TaskResult::default()
    }
}
