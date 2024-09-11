use mlua::prelude::LuaResult;
use mlua::Lua;
use std::path::PathBuf;

use crate::{diffuser::Diffuse, FuzzyTab};
use crate::NeoApi;

use super::{LineOut, NeoFuzzy, CONTAINER};

#[derive(Debug)]
pub struct SearchState {
    pub file_path: String,
    pub db_count: usize,
    pub update: bool,
    pub tabs: Vec<Box<dyn FuzzyTab>>,
    pub selected_tab: usize,
    pub selected_idx: usize,
}

impl SearchState {
    /// TODO increment or decrement
    pub async fn change_tab(lua: Lua, _: ()) -> LuaResult<()> {
        let mut state = CONTAINER.search_state.write().await;

        if state.selected_tab + 1 < state.tabs.len() {
            state.selected_tab += 1;
        } else {
            state.selected_tab = 0;
        }

        let selected_tab = state.selected_tab;
        let selected_idx = state.selected_idx;

        drop(state);

        let search_query = NeoApi::get_current_line(&lua)?;
        let fuzzy_c = &CONTAINER.fuzzy.read().await.config;

        Diffuse::queue([
            //Box::new(ClearResultsTask),
            fuzzy_c.search_task(&lua, search_query, selected_tab),
            fuzzy_c.preview_task(&lua, selected_idx, selected_tab),
        ])
        .await;

        Ok(())
    }

    pub fn get_selected(
        fuzzy: &NeoFuzzy,
        search_lines: &[LineOut],
        selected_idx: usize,
    ) -> PathBuf {
        fuzzy
            .config
            .cwd()
            .join(search_lines[selected_idx].text.as_ref())
    }
}
