// Some method to prevent deadlocking
// I prefer try_read / try_write for everything since its safe
// Maybe store some state so we can always determine whoever can go next
// Create Linked list with actions
// Try to lock do something then next

use std::{sync::LazyLock, time::Duration};
use tokio::{sync::Mutex, time};

use crate::RTM;

static DIFFUSER: LazyLock<Mutex<Diffuse>> = LazyLock::new(|| Diffuse::default().into());

#[derive(Default)]
pub struct Diffuse {
    run: bool,
    queue: Vec<Box<dyn ExecuteTask>>,
}

unsafe impl Send for Diffuse {}

#[async_trait::async_trait]
pub trait ExecuteTask: Send {
    async fn execute(&self);
}

/// Use it if you don't need a task
pub struct DummyTask;

#[async_trait::async_trait]
impl ExecuteTask for DummyTask {
    async fn execute(&self) {}
}


impl Diffuse {
    pub async fn queue(task_list: Vec<Box<dyn ExecuteTask>>) {
        let mut diffuser = DIFFUSER.lock().await;

        for new_task in task_list.into_iter() {
            diffuser.queue.push(new_task);
        }
    }

    pub async fn start() {
        let mut diffuser = DIFFUSER.lock().await;
        diffuser.run = true;

        //let _ = RTM.enter();
        RTM.spawn(async {
            let mut interval = time::interval(Duration::from_millis(1));

            loop {
                let mut diffuser = DIFFUSER.lock().await;

                if !diffuser.run && diffuser.queue.is_empty() {
                    break;
                }

                let queue = std::mem::take(&mut diffuser.queue);

                drop(diffuser);

                for current in queue {
                    current.execute().await;
                }

                interval.tick().await;
            }
        });
    }

    pub async fn stop() {
        DIFFUSER.lock().await.run = false;
    }
}
