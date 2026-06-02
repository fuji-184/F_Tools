
/*
Asynchronous thread-safe one-time initialization cell.

This structure guarantees that a shared resource or heavy dependency is initialized 
exactly once across concurrent execution flows. If multiple asynchronous tasks attempt 
to access uninitialized data simultaneously, they will safely suspend and wait for the 
first worker to complete the setup closure, thereafter resolving to the same shared reference.
*/

use tokio::sync::OnceCell;

pub struct AsyncOnce<T> {
    cell: OnceCell<T>,
}

impl<T> AsyncOnce<T> {
    pub fn new() -> Self {
        Self { cell: OnceCell::new() }
    }

    pub async fn get_or_init<F, Fut>(&self, init: F) -> &T
    where
        F: FnOnce() -> Fut,
        Fut: std::future::Future<Output = T>,
    {
        self.cell.get_or_init(init).await
    }
}

ftest::test!(async_once_tests, {
    test_initialization_once.tokio {
        let once = AsyncOnce::new();
        let counter = std::sync::atomic::AtomicUsize::new(0);
        let counter = std::sync::Arc::new(counter);

        let c1 = counter.clone();
        let f1 = once.get_or_init(|| async move {
            c1.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
            42
        });

        let c2 = counter.clone();
        let f2 = once.get_or_init(|| async move {
            c2.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
            99
        });

        let (res1, res2) = tokio::join!(f1, f2);

        assert_eq!(*res1, 42);
        assert_eq!(*res2, 42);
        assert_eq!(counter.load(std::sync::atomic::Ordering::SeqCst), 1);
    }

    test_concurrent_initialization.tokio {
        let once = std::sync::Arc::new(AsyncOnce::new());
        let mut handles = Vec::new();

        for i in 0..2 {
            let once_clone = once.clone();
            handles.push(tokio::spawn(async move {
                let val = once_clone.get_or_init(|| async move {
                    i
                }).await;
                *val
            }));
        }

        let mut results = Vec::new();
        for handle in handles {
            results.push(handle.await.unwrap());
        }

        assert_eq!(results[0], results[1]);
    }
});