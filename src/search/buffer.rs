use mlua::{prelude::LuaResult, Lua};
use std::{
    borrow::Cow,
    path::{Path, PathBuf},
};

use crate::{
    search::{FuzzyTab, TaskResult},
    web_devicons::DevIcon,
    BufInfo, BufInfoOpts, ExecuteTask, LineOut, NeoApi, NeoDebug, NeoUtils, CONTAINER,
};

use super::PreviewTask;

pub struct BufferSearch {
    /// Based on current tab
    pub search_query: String,
    pub buf_infos: Vec<BufInfo>,
    pub selected_tab: usize,
    pub cwd: PathBuf,
}

impl BufferSearch {
    pub fn new(lua: &Lua, cwd: &Path, selected_tab: usize) -> LuaResult<Self> {
        let search_query = NeoApi::get_current_line(lua)?;
        let buf_infos = NeoApi::get_buf_info(lua, BufInfoOpts::BufListed)?;

        Ok(Self {
            search_query,
            buf_infos,
            selected_tab,
            cwd: cwd.to_path_buf(),
        })
    }

    async fn init(&self) -> TaskResult {
        let mut new_lines = vec![];
        let mut tabs: Vec<Box<dyn FuzzyTab>> = vec![];

        let git_root = NeoUtils::git_root(&self.cwd);
        let mut push_other_tab = true;

        if let Some(git_root) = git_root {
            tabs.push(Box::new(git_root));
        } else {
            tabs.push(Box::new(" other ".to_string()));
            push_other_tab = false;
        }

        for buf_info in self.buf_infos.iter() {
            let buf_path: PathBuf = buf_info.name.as_str().into();
            let dev_icon = DevIcon::get_icon(&buf_path);

            if let Some(git_root) = NeoUtils::git_root(&buf_path) {
                let path_suffix: Box<str> = buf_path
                    .strip_prefix(&git_root)
                    .unwrap()
                    .to_string_lossy()
                    .into();

                let git_root_str: Box<str> = git_root.to_string_lossy().into();
                let tab = Box::new(git_root);

                new_lines.push(LineOut {
                    text: path_suffix,
                    icon: dev_icon.icon.into(),
                    hl_group: dev_icon.highlight.into(),
                    git_root: git_root_str,
                    line_nr: Some(buf_info.lnum),
                });

                if !tabs.iter().any(|t| t.full() == tab.full()) {
                    tabs.push(tab);
                }
            } else {
                new_lines.push(LineOut {
                    text: buf_path.to_string_lossy().into(),
                    icon: dev_icon.icon.into(),
                    hl_group: dev_icon.highlight.into(),
                    line_nr: Some(buf_info.lnum),
                    ..Default::default()
                });
            }
        }

        if push_other_tab {
            tabs.push(Box::new(" other ".to_string()));
        }

        if let Err(e) = CONTAINER.db.insert_all(&new_lines).await {
            NeoDebug::log_dbg(e).await;
        }

        let new_lines = CONTAINER
            .db
            .search_project_lines("", tabs[self.selected_tab].full())
            .await;

        let mut result = TaskResult {
            db_count: Some(new_lines.len()),
            tabs: Some(tabs),
            selected_tab: Some(0),
            selected_idx: Some(0),
            line_prefix: Some(new_lines[0].git_root.as_ref().into()),
            update: true,
            ..Default::default()
        };

        if !new_lines.is_empty() {
            let path_prefix: PathBuf = new_lines[0].git_root.as_ref().into();
            let path_suffix = new_lines[0].text.clone();
            let prev_result = PreviewTask::new(path_prefix, path_suffix).execute().await;

            result.preview_lines = prev_result.preview_lines;
            result.file_path = prev_result.file_path;
        }

        result.new_lines = Some(new_lines);
        result
    }

    async fn search(&self) -> TaskResult {
        let search_state = CONTAINER.search_state.read().await;

        let other_tab = search_state.tabs.len() - 1 == search_state.selected_tab;

        let tab = if other_tab {
            Cow::from("")
        } else {
            search_state.tabs[search_state.selected_tab].full()
        };

        let new_lines = CONTAINER
            .db
            .search_project_lines(&self.search_query, tab)
            .await;

        if !new_lines.is_empty() {
            let path_prefix: PathBuf = new_lines[0].git_root.as_ref().into();
            let path_suffix = new_lines[0].text.clone();
            let prev_result = PreviewTask::new(path_prefix, path_suffix).execute().await;

            TaskResult {
                update: true,
                selected_idx: Some(0),
                new_lines: Some(new_lines),
                preview_lines: prev_result.preview_lines,
                file_path: prev_result.file_path,
                ..Default::default()
            }
        } else {
            TaskResult {
                update: true,
                selected_idx: Some(0),
                new_lines: Some(new_lines),
                preview_lines: Some(vec![]),
                ..Default::default()
            }
        }
    }
}

#[async_trait::async_trait]
impl ExecuteTask for BufferSearch {
    async fn execute(&self) -> TaskResult {
        if self.all_lines_is_empty().await {
            self.init().await
        } else {
            self.search().await
        }
    }
}

pub struct RemoveBuffer {
    pub selected_line: Box<str>,
}

//#[async_trait::async_trait]
//impl ExecuteTask for RemoveBuffer {
//async fn execute(&self) -> TaskResult {}
//}
