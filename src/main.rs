use futures::executor::block_on;
use rust_async::TimerFuture;
use std::time::Duration;

// For the executor
use futures::{
    future::{BoxFuture, FutureExt},
    task::{waker_ref, ArcWake},
};

use std::{
    future::Future,
    sync::mpsc::{sync_channel, Receiver, SyncSender},
    sync::{Arc, Mutex},
    task::Context,
};

async fn hello_world() {
    println!("Async hello world!")
}

async fn a() {
    println!("A");
}

async fn b() {
    println!("B");
}

async fn c() {
    a().await;
    b().await;
}

async fn d() {
    futures::join!(c(), hello_world());
}

fn main() {
    block_on(d());
    let x = TimerFuture::new(Duration::from_secs(6));
    println!("Created the future, blocking");
    block_on(x);
    println!("Finished");

    // Using our executor
    let (executor, spawner) = new_executor_and_spawner();
    spawner.spawn(async {
        println!("Inside our async block");
        TimerFuture::new(Duration::new(2, 0)).await;
        println!("After timer future, inside the block");
        d().await
    });

    spawner.spawn(d());

    drop(spawner);  // so it closes the channel

    executor.run();  // Blocks until all tasks are run
}

/// Executor receives tasks off of a channel and runs them
struct Executor {
    ready_queue: Receiver<Arc<Task>>,
}

/// Spawner sends the tasks into the channel
struct Spawner {
    task_sender: SyncSender<Arc<Task>>,
}

/// A future that can reschedule itself to be polled by an `Executor`
/// i.e. the Task is the Waker
struct Task {
    future: Mutex<Option<BoxFuture<'static, ()>>>,

    /// Handle so it can place itself back onto the queue
    task_sender: SyncSender<Arc<Task>>,
}

fn new_executor_and_spawner() -> (Executor, Spawner) {
    const MAX_QUEUED_TASKS: usize = 10000;
    let (task_sender, ready_queue) = sync_channel(MAX_QUEUED_TASKS);
    (Executor { ready_queue }, Spawner { task_sender })
}

impl Spawner {
    fn spawn(&self, future: impl Future<Output = ()> + 'static + Send) {
        let future = future.boxed();
        let task = Arc::new(Task {
            future: Mutex::new(Some(future)),
            task_sender: self.task_sender.clone(),
        });
        self.task_sender.send(task).expect("too many tasks queued");
    }
}

impl ArcWake for Task {
    fn wake_by_ref(arc_self: &Arc<Self>) {
        let cloned = arc_self.clone();
        arc_self.task_sender.send(cloned).expect("too many tasks queued");
    }
}

impl Executor {
    fn run(&self) {
        while let Ok(task) = self.ready_queue.recv() {
            let mut future_slot = task.future.lock().unwrap();
            if let Some(mut future) = future_slot.take() {
                let waker = waker_ref(&task);
                let context = &mut Context::from_waker(&waker);
                if future.as_mut().poll(context).is_pending() {
                    *future_slot = Some(future);
                }
            }
        }
    }
}
