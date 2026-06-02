use tokio::sync::mpsc;
use tokio_util::sync::CancellationToken;
use std::sync::Arc;
use tokio::task::JoinHandle;
use tokio::time::{timeout, Duration};
use std::mem;

type Job = Box<dyn FnOnce() + Send + 'static>;

pub struct AsyncTaskPool {
    sender: Option<mpsc::Sender<Job>>,
    workers: Vec<JoinHandle<()>>,
    token: CancellationToken,
}

impl AsyncTaskPool {
    pub fn new(num_workers: usize, queue_size: usize, task_timeout: Option<Duration>) -> Self {
        let (tx, rx) = mpsc::channel::<Job>(queue_size);
        let token = CancellationToken::new();
        let rx = Arc::new(tokio::sync::Mutex::new(rx));
        let mut workers = Vec::with_capacity(num_workers);

        for _ in 0..num_workers {
            let rx_clone = Arc::clone(&rx);
            let worker_token = token.clone();

            let handle = tokio::spawn(async move {
                loop {
                    tokio::select! {
                        _ = worker_token.cancelled() => {
                            break;
                        }
                        job_opt = async {
                            let mut lock = rx_clone.lock().await;
                            lock.recv().await
                        } => {
                            if let Some(job) = job_opt {
                                let task_future = tokio::task::spawn_blocking(move || {
                                    job();
                                });

                                if let Some(limit) = task_timeout {
                                    let _ = timeout(limit, task_future).await;
                                } else {
                                    let _ = task_future.await;
                                }
                            } else {
                                break;
                            }
                        }
                    }
                }
            });
            workers.push(handle);
        }

        Self {
            sender: Some(tx),
            workers,
            token,
        }
    }

    pub async fn execute<F>(&self, task: F) -> Result<(), String>
    where
        F: FnOnce() + Send + 'static,
    {
        if self.token.is_cancelled() {
            return Err("Pool is shut down".to_string());
        }
        if let Some(sender) = &self.sender {
            sender
               .send(Box::new(task))
                .await
                .map_err(|_| "Failed to send task".to_string())
        } else {
          Err(String::from("Task pool is in shutdown process"))
        }
    }

    pub fn cancel(&self) {
        self.token.cancel();
    }

    pub async fn shutdown_gracefully(mut self) {
        self.token.cancel();
        self.sender.take();
        let workers = mem::take(&mut self.workers);
        for worker in workers {
            let _ = worker.await;
        }
    }
}

impl Drop for AsyncTaskPool {
    fn drop(&mut self) {
        self.token.cancel();
    }
}


ftest::test!(async_task_pool_tests, {
    test_pool_execution.tokio {
        let pool = AsyncTaskPool::new(2, 5, None);
        let counter = std::sync::Arc::new(std::sync::atomic::AtomicUsize::new(0));

        for _ in 0..3 {
            let c = counter.clone();
            let res = pool.execute(move || {
                c.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
            }).await;
            assert!(res.is_ok());
        }

        tokio::time::sleep(std::time::Duration::from_millis(50)).await;

        pool.shutdown_gracefully().await;
        assert_eq!(counter.load(std::sync::atomic::Ordering::SeqCst), 3);
    }

    test_pool_cancel.tokio {
        let pool = AsyncTaskPool::new(1, 2, None);
        pool.cancel();

        let res = pool.execute(|| {}).await;
        assert!(res.is_err());
    }
});