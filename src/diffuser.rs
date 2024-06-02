// Some method to prevent deadlocking
// I prefer try_read / try_write for everything since its safe
// Maybe store some state so we can always determine whoever can go next
// Create Linked list with actions
// Try to lock do something then next

use once_cell::sync::Lazy;
use std::{collections::VecDeque, future::Future, pin::Pin, sync::Arc, time::Duration};
use tokio::{sync::Mutex, time};

use crate::RTM;

static DIFFUSER: Lazy<Mutex<Diffuse>> = Lazy::new(|| Diffuse::default().into());

#[derive(Default)]
pub struct Diffuse {
    run: bool,
    queue: VecDeque<Box<dyn ExecuteChain>>,
}

unsafe impl Send for Diffuse {}
unsafe impl Sync for Diffuse {}

/// Will execute part and act like a chain where every part will use try_read / try_write
pub type ChainLink = Option<Box<dyn ExecuteChain>>; 

pub trait ExecuteChain: Send + Sync {
    fn try_execute(self: Box<Self>) -> Pin<Box<dyn Future<Output = ChainLink> + Send>>;
}

impl Diffuse {
    pub async fn queue(head: Box<dyn ExecuteChain>) {
        let mut diffuser = DIFFUSER.lock().await;

        diffuser.queue.push_back(head);
    }

    pub async fn start() {
        let mut diffuser = DIFFUSER.lock().await;
        diffuser.run = true;
        drop(diffuser);

        RTM.spawn(async {
            let mut interval = time::interval(Duration::from_millis(1));

            loop {
                let mut diffuser = DIFFUSER.lock().await;

                if !diffuser.run {
                    return;
                }

                if let Some(current) = diffuser.queue.pop_front() {
                    if let Some(next) = current.try_execute().await {
                        diffuser.queue.push_front(next);
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
