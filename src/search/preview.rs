use std::{
    cmp::Ordering,
    collections::HashSet,
    path::{Path, PathBuf},
    time::Instant,
};
use tokio::{fs, io};

use crate::{search::TaskResult, ExecuteTask, NeoDebug};

pub struct PreviewTask {
    path_prefix: PathBuf,
    path_suffix: Box<str>,
}

impl PreviewTask {
    pub fn new(path_prefix: PathBuf, path_suffix: Box<str>) -> Self {
        Self {
            path_prefix,
            path_suffix,
        }
    }
}

#[async_trait::async_trait]
impl ExecuteTask for PreviewTask {
    async fn execute(&self) -> TaskResult {
        let now = Instant::now();

        let path = self.path_prefix.join(self.path_suffix.as_ref());

        let result = if path.is_dir() {
            preview_directory(&path).await
        } else if path.is_file() {
            preview_file(&path).await
        } else {
            Ok(TaskResult::default())
        };

        let elapsed_ms = now.elapsed().as_millis();
        NeoDebug::log(format!("Elapsed preview: {}", elapsed_ms)).await;

        if let Ok(result) = result {
            result
        } else {
            TaskResult::default()
        }
    }
}

async fn preview_directory(path: &Path) -> io::Result<TaskResult> {
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

    Ok(TaskResult {
        preview_lines: Some(items),
        update: true,
        ..Default::default()
    })
}

async fn preview_file(path: &Path) -> io::Result<TaskResult> {
    let mut result = TaskResult {
        update: true,
        ..Default::default()
    };

    if !is_binary(path) {
        if let Ok(file) = fs::read_to_string(path).await {
            let mut lines = vec![];

            for line in file.lines() {
                lines.push(line.into());
            }

            result.preview_lines = Some(lines);
            result.file_path = Some(path.to_string_lossy().to_string().into_boxed_str());

            return Ok(result);
        }
    }

    result.file_path = Some("text".into());
    result.preview_lines = Some(vec!["> File is a binary".into()]);

    Ok(result)
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
