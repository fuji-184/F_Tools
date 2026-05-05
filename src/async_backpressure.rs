use tokio::sync::Semaphore;
use std::sync::Arc;

pub struct AsyncBackpressure {
    semaphore: Arc<Semaphore>,
}

impl AsyncBackpressure {
    pub fn new(max_concurrency: usize) -> Self {
        Self {
            semaphore: Arc::new(Semaphore::new(max_concurrency)),
        }
    }

    pub async fn run<F, Fut, T>(&self, mut logic: F) -> Fut::Output
    where
        F: FnMut() -> Fut,
        Fut: std::future::Future,
    {
        let _permit = self.semaphore.acquire().await.unwrap();
        logic().await
    }
}