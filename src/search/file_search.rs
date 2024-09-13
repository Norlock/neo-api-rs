use std::{path::PathBuf, time::Instant};

use tokio::process::Command;

use crate::{web_devicons::DevIcon, ExecuteTask, NeoDebug, TaskResult};

use super::{LineOut, CONTAINER};

pub struct ExecFileSearch {
    pub cmd: &'static str,
    pub cwd: PathBuf,
    pub args: Vec<&'static str>,
    pub search_query: String,
}

impl ExecFileSearch {
    async fn insert_into_db(&self) -> TaskResult {
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
                let path = PathBuf::from(line);
                let dev_icon = DevIcon::get_icon(&path);

                new_lines.push(LineOut {
                    text: line.into(),
                    icon: dev_icon.icon.into(),
                    hl_group: dev_icon.highlight.into(),
                    git_root: None,
                });
            }

            match CONTAINER.db.insert_all(&new_lines).await {
                Ok(_) => {
                    if let Ok(new_lines) = CONTAINER.db.search_lines("").await {
                        *CONTAINER.search_lines.write().await = new_lines;
                    }

                    return TaskResult {
                        db_count: Some(new_lines.len()),
                        selected_idx: Some(0),
                        selected_tab: Some(0),
                        tabs: Some(vec![]),
                        ..Default::default()
                    };
                }
                Err(err) => {
                    NeoDebug::log(err).await;
                }
            }
        }

        TaskResult::default()
    }

    async fn db_search(&self) -> TaskResult {
        if let Ok(lines) = CONTAINER.db.search_lines(&self.search_query).await {
            *CONTAINER.search_lines.write().await = lines;
        }

        TaskResult::default()
    }
}

#[async_trait::async_trait]
impl ExecuteTask for ExecFileSearch {
    async fn execute(&self) -> TaskResult {
        let instant = Instant::now();

        let result = if self.is_initial_search().await {
            self.insert_into_db().await
        } else {
            self.db_search().await
        };

        let elapsed_ms = instant.elapsed().as_millis();
        NeoDebug::log(format!("Elapsed file search: {}", elapsed_ms)).await;

        result
    }
}
