use mlua::{prelude::LuaResult, Lua};
use std::path::{Path, PathBuf};

use crate::{
    diffuser::{FuzzyTab, TaskResult},
    web_devicons::DevIcon,
    BufInfo, BufInfoOpts, ExecuteTask, LineOut, NeoApi, NeoDebug, NeoUtils, CONTAINER,
};

pub struct BufferSearch {
    /// Based on current tab
    pub git_root: Option<PathBuf>,
    pub search_query: String,
    pub buf_infos: Vec<BufInfo>,
    pub selected_tab: usize,
}

impl BufferSearch {
    pub fn new(lua: &Lua, cwd: &Path, selected_tab: usize) -> LuaResult<Self> {
        let search_query = NeoApi::get_current_line(lua)?;
        let buf_infos = NeoApi::get_buf_info(lua, BufInfoOpts::BufListed)?;

        Ok(Self {
            search_query,
            buf_infos,
            git_root: NeoUtils::git_root(cwd),
            selected_tab,
        })
    }

    async fn init(&self) -> TaskResult {
        let mut new_lines = vec![];
        let mut tabs: Vec<Box<dyn FuzzyTab>> = vec![];

        if let Some(git_root) = self.git_root.clone() {
            tabs.push(Box::new(
                git_root.file_name().unwrap().to_string_lossy().to_string(),
            ));
        }

        for buf_info in self.buf_infos.iter() {
            let path = PathBuf::from(&buf_info.name);
            let dev_icon = DevIcon::get_icon(&path);

            if let Some(git_root) = NeoUtils::git_root(&path) {
                let path_suffix: Box<str> = path
                    .strip_prefix(&git_root)
                    .unwrap()
                    .to_string_lossy()
                    .into();

                let tab = git_root.file_name().unwrap().to_string_lossy().to_string();
                let git_root: Box<str> = git_root.to_string_lossy().into();

                new_lines.push(LineOut {
                    text: path_suffix,
                    icon: dev_icon.icon.into(),
                    hl_group: dev_icon.highlight.into(),
                    git_root: Some(git_root),
                });

                if !tabs.iter().any(|t| t.name() == &tab) {
                    tabs.push(Box::new(tab));
                }
            } else {
                new_lines.push(LineOut {
                    text: path.to_string_lossy().into(),
                    icon: dev_icon.icon.into(),
                    hl_group: dev_icon.highlight.into(),
                    git_root: None,
                });
            }
        }

        tabs.push(Box::new(" other ".to_string()));

        if let Err(e) = CONTAINER.db.insert_all(&new_lines).await {
            NeoDebug::log_dbg(e).await;
        }

        let new_lines = CONTAINER.db.search_project_lines("", &self.git_root).await;

        NeoDebug::log_dbg(&new_lines).await;
        let db_count = new_lines.len();
        *CONTAINER.search_lines.write().await = new_lines;

        TaskResult {
            db_count: Some(db_count),
            tabs: Some(tabs),
            selected_tab: Some(0),
            selected_idx: Some(0),
            update: true,
        }
    }

    async fn search(&self) -> TaskResult {
        let search_state = CONTAINER.search_state.read().await;

        let other_tab = search_state.tabs.len() - 1 == search_state.selected_tab;

        let tab = if other_tab {
            None
        } else {
            let tab = format!("%{}", search_state.tabs[search_state.selected_tab].name());
            Some(tab.into())
        };

        let lines = CONTAINER
            .db
            .search_project_lines(&self.search_query, &tab)
            .await;

        *CONTAINER.search_lines.write().await = lines;

        TaskResult::default()
    }
}

#[async_trait::async_trait]
impl ExecuteTask for BufferSearch {
    async fn execute(&self) -> TaskResult {
        if self.is_initial_search().await {
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
