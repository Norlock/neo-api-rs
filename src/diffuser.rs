// Some method to prevent deadlocking
// I prefer try_read / try_write for everything since its safe
// Maybe store some state so we can always determine whoever can go next
// Create Linked list with actions
// Try to lock do something then next

use std::{fmt, sync::LazyLock, time::Duration};
use tokio::{sync::Mutex, time};

use crate::{CONTAINER, RTM};

static DIFFUSER: LazyLock<Mutex<Diffuse>> = LazyLock::new(|| Diffuse::default().into());

#[derive(Default)]
pub struct Diffuse {
    queue: Vec<Box<dyn ExecuteTask>>,
}

unsafe impl Send for Diffuse {}

pub trait FuzzyTab: Send + Sync {
    fn name(&self) -> &str;
}

impl fmt::Debug for dyn FuzzyTab {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.name())
    }
}

impl FuzzyTab for &str {
    fn name(&self) -> &str {
        self
    }
}

impl FuzzyTab for String {
    fn name(&self) -> &str {
        self
    }
}

//pub struct TabContainer(Box<str>);

//impl TabContainer {
//pub fn new<T: Into<Box<str>>>(val: T) -> Self {
//TabContainer(val.into())
//}
//}

//unsafe impl Send for TabContainer {}

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

    async fn is_initial_search(&self) -> bool {
        CONTAINER.search_state.read().await.db_count == 0
    }
}

impl Diffuse {
    pub async fn queue<const ARR_SIZE: usize>(task_list: [Box<dyn ExecuteTask>; ARR_SIZE]) {
        let mut diffuser = DIFFUSER.lock().await;

        for new_task in task_list {
            diffuser.queue.push(new_task);
        }

        Self::start().await;
    }

    pub async fn start() {
        RTM.spawn(async {
            let mut interval = time::interval(Duration::from_millis(1));

            loop {
                let mut diffuser = DIFFUSER.lock().await;

                if diffuser.queue.is_empty() {
                    break;
                }

                let queue = std::mem::take(&mut diffuser.queue);

                drop(diffuser);

                for current in queue {
                    let result = current.execute().await;

                    if result.has_changes() {
                        let mut search_state = CONTAINER.search_state.write().await;

                        if result.update {
                            search_state.update = true;
                        }

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
                            search_state.tabs = tabs;
                        }
                    }
                }

                interval.tick().await;
            }
        });
    }
}
