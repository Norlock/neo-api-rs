use std::path::PathBuf;

use crate::{web_devicons::icons_default::DevIcon, BufInfo, ExecuteTask, LineOut, NeoDebug, CONTAINER};

pub struct BufferSearch {
    pub cwd: PathBuf,
    pub buf_infos: Vec<BufInfo>,
}

#[async_trait::async_trait]
impl ExecuteTask for BufferSearch {
    async fn execute(&self) {
        NeoDebug::log_dbg(&self.buf_infos).await;

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

        let mut lines = CONTAINER.search_lines.write().await;
        *lines = new_lines;
    }
}
