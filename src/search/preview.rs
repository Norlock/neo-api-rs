use std::{
    cmp::Ordering,
    collections::HashSet,
    path::{Path, PathBuf},
    time::Instant,
};

use tokio::{fs, io};

use crate::{search::TaskResult, ExecuteTask, NeoDebug};

use super::CONTAINER;

pub struct ExecPreview {
    pub cwd: PathBuf,
    pub selected_idx: usize,
}

#[async_trait::async_trait]
impl ExecuteTask for ExecPreview {
    async fn execute(&self) -> TaskResult {
        let now = Instant::now();

        let path: PathBuf = {
            let filtered_lines = CONTAINER.search_lines.read().await;

            if filtered_lines.is_empty() {
                CONTAINER.preview.write().await.clear();
                //CONTAINER.search_state.write().await.update = true;
                return TaskResult {
                    update: true,
                    ..Default::default()
                };
            }

            self.cwd
                .join(filtered_lines[self.selected_idx].text.as_ref())
        };

        if path.is_dir() && preview_directory(&path).await.is_ok()
            || path.is_file() && preview_file(&path).await.is_ok()
        {
            let elapsed_ms = now.elapsed().as_millis();
            NeoDebug::log(format!("Elapsed preview: {}", elapsed_ms)).await;

            return TaskResult {
                update: true,
                ..Default::default()
            };
        }

        TaskResult::default()
    }
}

async fn preview_directory(path: &Path) -> io::Result<()> {
    let mut items = Vec::new();
    let mut dir = fs::read_dir(path).await?;

    while let Some(item) = dir.next_entry().await? {
        if let Ok(file_type) = item.file_type().await {
            let name = item.file_name().to_string_lossy().into();

            if file_type.is_dir() {
                items.push(format!("{name}/").into_boxed_str());
            } else {
                items.push(name);
            }
        }
    }

    items.sort_by(|a, b| {
        if a.ends_with('/') == b.ends_with('/') {
            a.cmp(b)
        } else if a.ends_with('/') {
            Ordering::Less
        } else {
            Ordering::Greater
        }
    });

    if items.is_empty() {
        items.push("> Empty directory".into());
    }

    *CONTAINER.preview.write().await = items;

    Ok(())
}

async fn preview_file(path: &Path) -> io::Result<()> {
    let file_path;

    async fn handle_binary() {
        let mut preview = CONTAINER.preview.write().await;
        *preview = vec!["> File is a binary".into()];
    }

    if is_binary(path) {
        file_path = "text".to_string();
        handle_binary().await;
    } else {
        // TODO check for ZIP and preview
        if let Ok(file) = fs::read_to_string(path).await {
            file_path = path.to_string_lossy().to_string();

            let mut lines = vec![];

            for line in file.lines() {
                lines.push(line.into());
            }

            *CONTAINER.preview.write().await = lines;
        } else {
            file_path = "text".to_string();
            handle_binary().await;
        }
    }

    let mut search_state = CONTAINER.search_state.write().await;
    search_state.file_path = file_path;

    Ok(())
}

fn is_binary(file: &Path) -> bool {
    let binaries: HashSet<&str> = [
        "bin", "so", "mkv", "mp4", "blend", "jpg", "png", "jpeg", "webp",
    ]
    .into();

    if let Some(ext) = file.extension() {
        binaries.contains(ext.to_str().unwrap())
    } else {
        false
    }
}
