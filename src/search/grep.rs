use std::path::PathBuf;

use tokio::{process::Command, time::Instant};

use crate::NeoDebug;

use super::{ExecuteTask, TaskResult};

pub struct GrepTask {
    search_query: Box<str>,
    cwd: PathBuf,
}

#[async_trait::async_trait]
impl ExecuteTask for GrepTask {
    async fn execute(&self, instant: &Instant) -> TaskResult {
        let before_ms = instant.elapsed().as_millis();

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

        let after_ms = instant.elapsed().as_millis();

        NeoDebug::log(format!("Elapsed grep search: {}", after_ms - before_ms)).await;

        TaskResult {
            update: true,
            ..Default::default()
        }
    }
}
