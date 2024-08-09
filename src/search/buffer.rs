use std::path::{Path, PathBuf};

use mlua::{prelude::LuaResult, Lua};

use crate::{
    web_devicons::icons_default::DevIcon, BufInfo, BufInfoOpts, ExecuteTask, LineOut, NeoApi,
    NeoDebug, CONTAINER,
};

pub struct BufferSearch {
    pub cwd: PathBuf,
    pub search_query: String,
    pub buf_infos: Vec<BufInfo>,
}

impl BufferSearch {
    pub fn new(lua: &Lua, cwd: &Path) -> LuaResult<Self> {
        let search_query = NeoApi::get_current_line(lua)?;
        let buf_infos = NeoApi::get_buf_info(lua, BufInfoOpts::BufListed)?;

        Ok(Self {
            search_query,
            cwd: cwd.to_owned(),
            buf_infos,
        })
    }

    async fn init(&self) {
        let mut new_lines = vec![];

        for buf_info in self.buf_infos.iter() {
            let path = PathBuf::from(&buf_info.name);
            let dev_icon = DevIcon::get_icon(&path);

            if let Ok(path) = path.strip_prefix(&self.cwd) {
                new_lines.push(LineOut {
                    text: path.to_string_lossy().into(),
                    icon: dev_icon.icon.into(),
                    hl_group: dev_icon.highlight.into(),
                });
            } else {
                new_lines.push(LineOut {
                    text: path.to_string_lossy().into(),
                    icon: dev_icon.icon.into(),
                    hl_group: dev_icon.highlight.into(),
                });
            }
        }

        if let Err(e) = CONTAINER.db.insert_all(&new_lines).await {
            NeoDebug::log_dbg(e).await;
        }

        let count = new_lines.len();
        let mut lines = CONTAINER.search_lines.write().await;
        *lines = new_lines;

        let mut search_state = CONTAINER.search_state.write().await;
        search_state.db_count = count;
        search_state.update = true;
    }

    async fn search(&self) {
        if let Ok(lines) = CONTAINER.db.select(&self.search_query).await {
            *CONTAINER.search_lines.write().await = lines;
            CONTAINER.search_state.write().await.update = true;
        }
    }
}

#[async_trait::async_trait]
impl ExecuteTask for BufferSearch {
    async fn execute(&self) {
        if is_initial_search().await {
            self.init().await;
        } else {
            self.search().await;
        }
    }
}

async fn is_initial_search() -> bool {
    CONTAINER.search_state.read().await.db_count == 0
}

pub struct RemoveBuffer {
    pub selected_line: Box<str>,
}

#[async_trait::async_trait]
impl ExecuteTask for RemoveBuffer {
    async fn execute(&self) {}
}
