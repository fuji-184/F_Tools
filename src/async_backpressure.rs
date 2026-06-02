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

ftest::test!(async_backpressure_tests, {
    test_concurrency_limit.tokio {
        let backpressure = AsyncBackpressure::new(2);
        let counter = std::sync::atomic::AtomicUsize::new(0);
        let counter = Arc::new(counter);

        let c1 = counter.clone();
        let b1 = &backpressure;
        let f1 = b1.run::<_, _, ()>(move || {
            let c = c1.clone();
            async move {
                c.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
                tokio::time::sleep(std::time::Duration::from_millis(50)).await;
            }
        });

        let c2 = counter.clone();
        let b2 = &backpressure;
        let f2 = b2.run::<_, _, ()>(move || {
            let c = c2.clone();
            async move {
                c.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
                tokio::time::sleep(std::time::Duration::from_millis(50)).await;
            }
        });

        let c3 = counter.clone();
        let b3 = &backpressure;
        let f3 = b3.run::<_, _, ()>(move || {
            let c = c3.clone();
            async move {
                c.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
            }
        });

        let (join_res, _) = tokio::join!(
            async {
                tokio::time::sleep(std::time::Duration::from_millis(10)).await;
                let current = counter.load(std::sync::atomic::Ordering::SeqCst);
                assert_eq!(current, 2);
            },
            async {
                tokio::join!(f1, f2, f3);
            }
        );

        let _ = join_res;
        assert_eq!(counter.load(std::sync::atomic::Ordering::SeqCst), 3);
    }

    test_execution_order_and_output.tokio {
        let backpressure = AsyncBackpressure::new(1);
        let result = backpressure.run::<_, _, i32>(|| async {
            42
        }).await;
        assert_eq!(result, 42);
    }
});