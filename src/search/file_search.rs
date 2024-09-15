use std::{path::PathBuf, time::Instant};

use tokio::process::Command;

use crate::{web_devicons::DevIcon, ExecuteTask, NeoDebug, TaskResult};

use super::{LineOut, PreviewTask, CONTAINER};

pub struct ExecFileSearch {
    pub cmd: &'static str,
    pub cwd: PathBuf,
    pub args: Vec<&'static str>,
    pub search_query: String,
}

impl ExecFileSearch {
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
                let path = PathBuf::from(line);
                let dev_icon = DevIcon::get_icon(&path);

                new_lines.push(LineOut {
                    text: line.into(),
                    icon: dev_icon.icon.into(),
                    hl_group: dev_icon.highlight.into(),
                    ..Default::default()
                });
            }

            match CONTAINER.db.insert_all(&new_lines).await {
                Ok(_) => {
                    let db_count = new_lines.len();

                    if let Ok(new_lines) = CONTAINER.db.search_lines("").await {
                        let prev_result =
                            PreviewTask::new(self.cwd.clone(), new_lines[0].text.clone())
                                .execute()
                                .await;

                        return TaskResult {
                            db_count: Some(db_count),
                            selected_idx: Some(0),
                            selected_tab: Some(0),
                            line_prefix: Some(self.cwd.clone()),
                            line_nr: Some(0),
                            tabs: Some(vec![]),
                            update: true,
                            preview_lines: prev_result.preview_lines,
                            file_path: prev_result.file_path,
                            new_lines: Some(new_lines),
                        };
                    }
                }
                Err(err) => {
                    NeoDebug::log(err).await;
                }
            }
        }

        TaskResult::default()
    }

    async fn db_search(&self) -> TaskResult {
        if let Ok(new_lines) = CONTAINER.db.search_lines(&self.search_query).await {
            let prev_result = if !new_lines.is_empty() {
                let path_prefix = self.cwd.clone();
                let path_suffix = new_lines[0].text.clone();

                PreviewTask::new(path_prefix, path_suffix).execute().await
            } else {
                TaskResult::default()
            };

            return TaskResult {
                update: true,
                preview_lines: prev_result.preview_lines,
                file_path: prev_result.file_path,
                new_lines: Some(new_lines),
                ..Default::default()
            };
        }

        TaskResult::default()
    }
}

#[async_trait::async_trait]
impl ExecuteTask for ExecFileSearch {
    async fn execute(&self) -> TaskResult {
        let instant = Instant::now();

        let result = if self.all_lines_is_empty().await {
            self.insert_into_db().await
        } else {
            self.db_search().await
        };

        let elapsed_ms = instant.elapsed().as_millis();
        NeoDebug::log(format!("Elapsed file search: {}", elapsed_ms)).await;

        result
    }
}
