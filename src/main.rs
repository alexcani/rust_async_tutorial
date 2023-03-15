use futures::executor::block_on;

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
    block_on(d())
}
