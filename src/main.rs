use std::pin::Pin;
use std::sync::{Arc, Mutex};
use std::task::Poll;
use std::task::Waker;
use std::thread;
use std::time::Duration;

use futures::executor::block_on;
use futures::Future;

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
}

pub struct TimerFuture {
    shared_state: Arc<Mutex<SharedState>>,
}

struct SharedState {
    completed: bool,
    waker: Option<Waker>,
}

impl Future for TimerFuture {
    type Output = ();
    fn poll(self: Pin<&mut Self>, cx: &mut std::task::Context<'_>) -> Poll<Self::Output> {
        let mut shared_state = self.shared_state.lock().unwrap();
        if shared_state.completed {
            Poll::Ready(())
        } else {
            shared_state.waker = Some(cx.waker().clone());
            Poll::Pending
        }
    }
}

impl TimerFuture {
    pub fn new(duration: Duration) -> Self {
        let shared_state = Arc::new(Mutex::new(SharedState {
            completed: false,
            waker: None,
        }));

        let thread_shared_state = shared_state.clone();
        thread::spawn(move || {
            thread::sleep(duration);
            let mut shared_state = thread_shared_state.lock().unwrap();
            shared_state.completed = true;
            println!("Done sleeping");
            if let Some(waker) = shared_state.waker.take() {
                println!("Waking up");
                waker.wake();
            }
        });

        TimerFuture { shared_state }
    }
}
