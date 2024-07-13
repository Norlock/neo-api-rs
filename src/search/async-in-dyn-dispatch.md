# How to easily create an async task queue using dynamic dispatch
This document hopefully can help people implement async traits in dynamic dispatched objects without too much hazzle.
I have seen a lot of the following constructs: 
```rust
Box::new(move |a, b| async move some_func(a,b))
```

Which is not very flexible code, because this requires that all the tasks to contain the same arguments. So I came up with something which I thought was interesting enough to share. A little bit of context:

For this project I'm making a plugin for Rust that will work with Neovim. 
I was busy with a fuzzy file searcher that would spawn async tasks on `text-change` events.

I decided to make a queue so that all those async tasks would run serial instead of parallel to avoid potential deadlocks and avoid hammering the cpu and memory. 
The async task trait has one function `execute` that would run something asynchronous. Since Rust 1.75 
the standard library allows async fn to be used in traits: https://github.com/rust-lang/rust/pull/115822/. This would only work on none `dyn dispatched` objects, that means the following would work:

```rust
struct TestComp {
    text: String,
}

impl AsyncTask for TestComp {
    async fn execute(&self) {
        println!("hello");
    }
}

trait AsyncTask: Send {
    async fn execute(&self);
}

async fn test() {
    // Store as a type!
    let a = TestComp {
        text: "asd".to_string(),
    };

    a.execute().await;
}
```

But this wouldn't compile:

```rust
struct TestComp {
    text: String,
}

impl AsyncTask for TestComp {
    async fn execute(&self) {
        println!("hello");
    }
}

trait AsyncTask: Send {
    async fn execute(&self);
}

async fn test() {
    // Store as dynamic dispatch!
    let a: Box<dyn AsyncTask> = Box::new(TestComp {
      text: "asd".to_string(),
    });

    a.execute().await;
}
```

So this would make it impossible to compile the following code: 

```rust
struct Diffuser {
    tasks: Vec<Box<dyn AsyncTask>>,
}

impl Diffuser {
    fn run(&self) {
        for task in self.tasks.iter() {
            task.execute();
        }
    }
}
```

However using pin and box together you can make async work again:

```rust
static DIFFUSER: Lazy<Mutex<Diffuse>> = Lazy::new(|| Diffuse::default().into());

#[derive(Default)]
pub struct Diffuse {
    run: bool,
    queue: Vec<Box<dyn ExecuteTask>>,
}

unsafe impl Send for Diffuse {}

/// Will execute part and act like a chain where every part will use try_read / try_write
pub type TaskResult<'a> = Pin<Box<dyn Future<Output = ()> + Send + 'a>>;

pub trait ExecuteTask: Send {
    fn execute<'a>(&'a self) -> TaskResult<'a>;
}

impl Diffuse {
    pub async fn queue<const N: usize>(task_list: [Box<dyn ExecuteTask>; N]) {
        let mut diffuser = DIFFUSER.lock().await;

        for new_task in task_list.into_iter() {
            diffuser.queue.push(new_task);
        }
    }

    pub async fn start() {
        let mut diffuser = DIFFUSER.lock().await;
        diffuser.run = true;

        let _ = RTM.enter();
        RTM.spawn(async {
            let mut interval = time::interval(Duration::from_millis(1));

            loop {
                let mut diffuser = DIFFUSER.lock().await;

                if !diffuser.run {
                    break;
                }

                let to_exec = std::mem::take(&mut diffuser.queue);

                drop(diffuser);

                for current in to_exec {
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
```

This trait can for example be implemented as following:

```rust
struct ExecPreview {
    cwd: PathBuf,
    selected_idx: usize,
}

impl ExecPreview {
    async fn run(&self) {
        let now = Instant::now();

        let path: PathBuf = {
            let filtered_lines = CONTAINER.sorted_lines.read().await;

            if filtered_lines.is_empty() {
                CONTAINER.preview.write().await.clear();
                CONTAINER.search_state.write().await.update = true;
                return;
            }

            self.cwd
                .join(filtered_lines[self.selected_idx].text.as_ref())
        };

        if path.is_dir() && preview_directory(&path).await.is_ok()
            || path.is_file() && preview_file(&path).await.is_ok()
        {
            CONTAINER.search_state.write().await.update = true;
            let elapsed_ms = now.elapsed().as_millis();
            NeoDebug::log(format!("elapsed preview: {}", elapsed_ms)).await;
        }
    }
}

impl ExecuteTask for ExecPreview {
    fn execute<'a>(&'a self) -> TaskResult<'a> {
        Box::pin(self.run())
    }
}
```

That's all, you can create all kind of task objects with different parameters. Hopefully this trick simplified async traits in dynamically dispatched objects for you. 
