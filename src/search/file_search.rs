use std::path::PathBuf;
use tokio::{process::Command, time::Instant};

use crate::{web_devicons::DevIcon, ExecuteTask, NeoDebug, TaskResult};

use super::{LineOut, Preview, CONTAINER};

pub struct FileSearchTask {
    pub cmd: &'static str,
    pub cwd: PathBuf,
    pub args: Vec<&'static str>,
    pub search_query: String,
}

impl FileSearchTask {
    async fn insert_into_db(&self, instant: &Instant) -> TaskResult {
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
                    path_suffix: line.into(),
                    icon: dev_icon.icon.into(),
                    hl_group: dev_icon.highlight.into(),
                    path_prefix: self.cwd.to_string_lossy().into(),
                    line_nr: 1,
                });
            }

            match CONTAINER.db.insert_all(&new_lines).await {
                Ok(_) => {
                    let db_count = new_lines.len();

                    if let Ok(new_lines) = CONTAINER.db.search_lines("").await {
                        let preview_lines = if new_lines.is_empty() {
                            Some(vec![])
                        } else {
                            Some(Preview::get_lines(new_lines[0].clone(), instant).await)
                        };

                        return TaskResult {
                            db_count: Some(db_count),
                            selected_idx: Some(0),
                            selected_tab: Some(0),
                            tabs: Some(vec![]),
                            search_lines: Some(new_lines),
                            preview_lines,
                            update: true,
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

    async fn db_search(&self, instant: &Instant) -> TaskResult {
        if let Ok(new_lines) = CONTAINER.db.search_lines(&self.search_query).await {
            let preview_lines = if new_lines.is_empty() {
                Some(vec![])
            } else {
                Some(Preview::get_lines(new_lines[0].clone(), instant).await)
            };

            return TaskResult {
                update: true,
                preview_lines,
                search_lines: Some(new_lines),
                ..Default::default()
            };
        }

        TaskResult::default()
    }
}

#[async_trait::async_trait]
impl ExecuteTask for FileSearchTask {
    async fn execute(&self, instant: &Instant) -> TaskResult {
        let before_ms = instant.elapsed().as_millis();

        let result = if self.all_lines_is_empty().await {
            self.insert_into_db(instant).await
        } else {
            self.db_search(instant).await
        };

        let after_ms = instant.elapsed().as_millis();

        NeoDebug::log(format!("Elapsed file search: {}", after_ms - before_ms)).await;

        result
    }
}
