/// Output from timing a process
pub struct Timing {}

pub fn timed<U, F: FnOnce() -> U>(f: F) -> (U, std::time::Duration) {
    use std::time::Instant;
    let start = Instant::now();
    let out = f();
    let end = Instant::now();
    let elapsed = end - start;
    (out, elapsed)
}

pub async fn timed_async<U, O: std::future::Future<Output = U>, F: FnOnce() -> O>(
    f: F,
) -> (U, std::time::Duration) {
    use std::time::Instant;
    let start = Instant::now();
    let out = f().await;
    let end = Instant::now();
    let elapsed = end - start;
    (out, elapsed)
}
