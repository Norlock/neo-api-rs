use std::{
    cmp::Ordering,
    collections::HashSet,
    path::Path,
    time::Instant,
};
use tokio::fs;

use crate::NeoDebug;

use super::LineOut;

pub struct Preview;

impl Preview {
    pub async fn get_lines(line_out: LineOut) -> Vec<Box<str>> {
        let now = Instant::now();

        let path = line_out.full_path_buf();

        let result = if path.is_dir() {
            preview_directory(&path).await
        } else if path.is_file() {
            preview_file(&path).await
        } else {
            vec![]
        };

        let elapsed_ms = now.elapsed().as_millis();
        NeoDebug::log(format!("Elapsed preview: {}", elapsed_ms)).await;

        result
    }
}

async fn preview_directory(path: &Path) -> Vec<Box<str>> {
    let mut items = Vec::new();
    let mut dir = fs::read_dir(path).await.unwrap();

    while let Ok(Some(item)) = dir.next_entry().await {
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

    items
}

async fn preview_file(path: &Path) -> Vec<Box<str>> {
    if !is_binary(path) {
        if let Ok(file) = fs::read_to_string(path).await {
            let mut lines = vec![];

            for line in file.lines() {
                lines.push(line.into());
            }

            lines
        } else {
            vec![]
        }
    } else {
        vec!["> File is a binary".into()]
    }
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
