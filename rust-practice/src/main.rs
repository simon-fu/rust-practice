


pub mod lock;

#[cfg(test)]
mod log_tracing_span;

#[tokio::main]
async fn main() {
    println!("Hello, world!");
    lock::test_dead_lock2();
}
