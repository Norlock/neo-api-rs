use mlua::{prelude::LuaResult, Lua};
use std::path::{Path, PathBuf};

use crate::{
    search::{FuzzyTab, TaskResult},
    web_devicons::DevIcon,
    BufInfo, BufInfoOpts, ExecuteTask, LineOut, NeoApi, NeoDebug, NeoUtils, CONTAINER,
};

use super::Preview;

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

                let path_prefix: Box<str> = git_root.to_string_lossy().into();
                let tab = Box::new(git_root);

                new_lines.push(LineOut {
                    path_prefix,
                    path_suffix,
                    icon: dev_icon.icon.into(),
                    hl_group: dev_icon.highlight.into(),
                    line_nr: buf_info.lnum,
                });

                if !tabs.iter().any(|t| t.full() == tab.full()) {
                    tabs.push(tab);
                }
            } else {
                new_lines.push(LineOut {
                    path_prefix: "".into(),
                    path_suffix: buf_path.to_string_lossy().into(),
                    icon: dev_icon.icon.into(),
                    hl_group: dev_icon.highlight.into(),
                    line_nr: buf_info.lnum,
                });
            }
        }

        if push_other_tab {
            tabs.push(Box::new(" other ".to_string()));
        }

        let db_count = new_lines.len();

        if let Err(e) = CONTAINER.db.insert_all(&new_lines).await {
            NeoDebug::log_dbg(e).await;
        }

        let new_lines = CONTAINER.db.search_project_lines("", &tabs[0].full()).await;

        let preview_lines = if new_lines.is_empty() {
            vec![]
        } else {
            Preview::get_lines(new_lines[0].clone()).await
        };

        TaskResult {
            db_count: Some(db_count),
            tabs: Some(tabs),
            selected_tab: Some(0),
            selected_idx: Some(0),
            search_lines: Some(new_lines),
            preview_lines: Some(preview_lines),
            update: true,
            ..Default::default()
        }
    }

    async fn search(&self) -> TaskResult {
        let search_state = CONTAINER.search_state.read().await;
        let tab = search_state.tabs[search_state.selected_tab].full();

        let new_lines = CONTAINER
            .db
            .search_project_lines(&self.search_query, &tab)
            .await;

        let preview_lines = if new_lines.is_empty() {
            vec![]
        } else {
            Preview::get_lines(new_lines[0].clone()).await
        };

        TaskResult {
            update: true,
            selected_idx: Some(0),
            search_lines: Some(new_lines),
            preview_lines: Some(preview_lines),
            ..Default::default()
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
