// Some method to prevent deadlocking
// I prefer try_read / try_write for everything since its safe
// Maybe store some state so we can always determine whoever can go next
// Create Linked list with actions
// Try to lock do something then next

use once_cell::sync::Lazy;
use std::{collections::VecDeque, future::Future, pin::Pin, time::Duration};
use tokio::{sync::Mutex, time};

use crate::{NeoDebug, RTM};

static DIFFUSER: Lazy<Mutex<Diffuse>> = Lazy::new(|| Diffuse::default().into());

#[derive(Default)]
pub struct Diffuse {
    run: bool,
    queue: VecDeque<Box<dyn ExecuteTask>>,
}

unsafe impl Send for Diffuse {}
unsafe impl Sync for Diffuse {}

/// Will execute part and act like a chain where every part will use try_read / try_write
pub type ChainLink = Option<Box<dyn ExecuteTask>>;
pub type ChainResult = Pin<Box<dyn Future<Output = ChainLink> + Send>>;

pub trait ExecuteTask: Send {
    fn try_execute(self: Box<Self>) -> ChainResult;
    fn failure_count(&mut self) -> &mut usize;
    fn id(&self) -> &str;
}

impl Diffuse {
    pub async fn queue<const N: usize>(task_list: [Box<dyn ExecuteTask>; N]) {
        let mut diffuser = DIFFUSER.lock().await;

        for new_task in task_list.into_iter() {
            diffuser.queue.push_back(new_task);
        }
    }

    pub async fn start() {
        let mut diffuser = DIFFUSER.lock().await;
        diffuser.run = true;

        RTM.spawn(async {
            let mut interval = time::interval(Duration::from_millis(1));

            loop {
                let mut diffuser = DIFFUSER.lock().await;

                if !diffuser.run {
                    return;
                }

                if let Some(current) = diffuser.queue.pop_front() {
                    if let Some(mut next) = current.try_execute().await {
                        *next.failure_count() += 1;
                        let failure_count = *next.failure_count();

                        NeoDebug::log(format!(
                            "Task: {} failed {} times.",
                            next.id(),
                            failure_count
                        ))
                        .await;

                        if *next.failure_count() < 100 {
                            diffuser.queue.push_front(next);
                        }
                    }
                }

                drop(diffuser);
                interval.tick().await;
            }
        });
    }

    pub async fn stop() {
        DIFFUSER.lock().await.run = false;
    }
}
