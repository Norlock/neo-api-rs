// Some method to prevent deadlocking
// I prefer try_read / try_write for everything since its safe
// Maybe store some state so we can always determine whoever can go next
// Create Linked list with actions
// Try to lock do something then next

use std::{borrow::Cow, fmt, path::PathBuf, sync::LazyLock};
use tokio::sync::Mutex;

use crate::{NeoDebug, CONTAINER, RTM};

static DIFFUSER: LazyLock<Mutex<Diffuse>> = LazyLock::new(|| Diffuse::default().into());

#[derive(Default)]
pub struct Diffuse {
    queue: Vec<Box<dyn ExecuteTask>>,
    is_running: bool,
}

unsafe impl Send for Diffuse {}

pub trait FuzzyTab: Send + Sync {
    fn name(&self) -> Cow<'_, str>;

    fn full(&self) -> Cow<'_, str>;
}

impl fmt::Debug for dyn FuzzyTab {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.name())
    }
}

impl FuzzyTab for &str {
    fn name(&self) -> Cow<'_, str> {
        String::from_utf8_lossy(self.as_bytes())
    }

    fn full(&self) -> Cow<'_, str> {
        String::from_utf8_lossy(self.as_bytes())
    }
}

impl FuzzyTab for String {
    fn name(&self) -> Cow<'_, str> {
        String::from_utf8_lossy(self.as_bytes())
    }

    fn full(&self) -> Cow<'_, str> {
        String::from_utf8_lossy(self.as_bytes())
    }
}

impl FuzzyTab for PathBuf {
    fn name(&self) -> Cow<'_, str> {
        self.file_name().expect("Has no filename").to_string_lossy()
    }

    fn full(&self) -> Cow<'_, str> {
        self.to_string_lossy()
    }
}

#[derive(Default)]
pub struct TaskResult {
    pub db_count: Option<usize>,
    pub selected_idx: Option<usize>,
    pub selected_tab: Option<usize>,
    pub tabs: Option<Vec<Box<dyn FuzzyTab>>>,
    pub update: bool,
}

impl TaskResult {
    pub fn has_changes(&self) -> bool {
        self.update
            || self.db_count.is_some()
            || self.selected_idx.is_some()
            || self.selected_tab.is_some()
            || self.tabs.is_some()
    }
}

#[async_trait::async_trait]
pub trait ExecuteTask: Send {
    async fn execute(&self) -> TaskResult;

    async fn all_lines_is_empty(&self) -> bool {
        CONTAINER.db.all_lines_is_empty().await
    }
}

impl Diffuse {
    pub async fn queue<const ARR_SIZE: usize>(task_list: [Box<dyn ExecuteTask>; ARR_SIZE]) {
        let mut diffuser = DIFFUSER.lock().await;

        for new_task in task_list {
            diffuser.queue.push(new_task);
        }

        if !diffuser.is_running {
            diffuser.is_running = true;
            Self::start();
        }
    }

    pub fn start() {
        RTM.spawn(async {
            loop {
                let mut diffuser = DIFFUSER.lock().await;

                if diffuser.queue.is_empty() {
                    diffuser.is_running = false;
                    return;
                }

                let queue = std::mem::take(&mut diffuser.queue);

                drop(diffuser);

                for current in queue {
                    let result = current.execute().await;

                    if result.has_changes() {
                        let mut search_state = CONTAINER.search_state.write().await;

                        if let Some(db_count) = result.db_count {
                            search_state.db_count = db_count;
                        }

                        if let Some(selected_idx) = result.selected_idx {
                            search_state.selected_idx = selected_idx;
                        }

                        if let Some(selected_tab) = result.selected_tab {
                            search_state.selected_tab = selected_tab;
                        }

                        if let Some(tabs) = result.tabs {
                            NeoDebug::log_dbg(&tabs).await;
                            search_state.tabs = tabs;
                        }

                        if result.update {
                            search_state.update = true;
                        }
                    }
                }
            }
        });
    }
}
