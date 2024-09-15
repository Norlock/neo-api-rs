use std::{path::PathBuf, time::Instant};

use tokio::process::Command;

use super::{ExecuteTask, TaskResult};

pub struct GrepTask {
    search_query: Box<str>,
    cwd: PathBuf,
}

#[async_trait::async_trait]
impl ExecuteTask for GrepTask {
    async fn execute(&self) -> TaskResult {
        let instant = Instant::now();

        let out = Command::new("rg")
            .args([&self.search_query, "--no-heading"])
            .current_dir(&self.cwd)
            .output()
            .await
            .unwrap();

        if out.status.success() {
            let lines = String::from_utf8_lossy(&out.stdout);

            for line in lines.lines() {
                let parts = line.splitn(3, ':');

                for part in parts {

                }
                //if let Some( ) = line.splitn(':');
            }
        }

        instant.elapsed();

        TaskResult {
            update: true,
            ..Default::default()
        }
    }
}
