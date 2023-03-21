use futures::executor::block_on;
use std::time::Duration;
use rust_async::TimerFuture;

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
