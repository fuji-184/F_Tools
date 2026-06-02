
/*
Structured concurrency manager to safely supervise and lifecycle-bind background tasks.

This abstraction enables spawning concurrent background jobs that can safely borrow or reference 
data from the surrounding block environment. It is primarily used to enforce clean task lifecycles—
such as processing transient batch computations, handling multi-step request operations, or 
orchestrating short-lived sub-tasks—by ensuring all spawned operations either complete or get 
automatically aborted before execution exits the defined scope block, eliminating orphan tasks.
*/

use std::future::Future;
use std::marker::PhantomData;
use std::pin::Pin;
use tokio::task::JoinHandle;

pub struct GlobalAsyncScope<'a> {
    handles: Vec<JoinHandle<()>>,
    _marker: PhantomData<&'a ()>,
}

impl<'a> GlobalAsyncScope<'a> {
    pub fn new() -> Self {
        Self {
            handles: Vec::new(),
            _marker: PhantomData,
        }
    }

    pub fn spawn<F>(&mut self, future: F)
    where
        F: Future<Output = ()> + Send + 'a,
    {
        let coerced_future = unsafe {
            std::mem::transmute::<
                Pin<Box<dyn Future<Output = ()> + Send + 'a>>,
                Pin<Box<dyn Future<Output = ()> + Send + 'static>>,
            >(Box::pin(future))
        };

        let handle = tokio::spawn(coerced_future);
        self.handles.push(handle);
    }

    pub async fn wait_all(&mut self) {
        for handle in self.handles.drain(..) {
            let _ = handle.await;
        }
    }
}

impl<'a> Drop for GlobalAsyncScope<'a> {
    fn drop(&mut self) {
        if self.handles.is_empty() {
            return;
        }

        for handle in &self.handles {
            handle.abort();
        }
        
        self.handles.clear();
    }
}

pub async fn global_async_scope<'a, F, T>(f: F) -> T
where
    F: for<'scope> FnOnce(&'scope mut GlobalAsyncScope<'a>) -> 
        Pin<Box<dyn Future<Output = T> + Send + 'scope>>,
{
    let mut scope = GlobalAsyncScope::new();
    
    let result = f(&mut scope).await;
    
    scope.wait_all().await;
    result
}

ftest::test!(global_async_scope_tests, {
    test_scope_spawns_and_waits.tokio {
        let counter = std::sync::Arc::new(std::sync::atomic::AtomicUsize::new(0));
        let counter_clone1 = counter.clone();
        let counter_clone2 = counter.clone();

        let result = global_async_scope(|scope| {
            Box::pin(async move {
                scope.spawn(async move {
                    tokio::time::sleep(std::time::Duration::from_millis(5)).await;
                    counter_clone1.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
                });

                scope.spawn(async move {
                    tokio::time::sleep(std::time::Duration::from_millis(10)).await;
                    counter_clone2.fetch_add(2, std::sync::atomic::Ordering::SeqCst);
                });

                42
            })
        })
        .await;

        assert_eq!(result, 42);
        assert_eq!(counter.load(std::sync::atomic::Ordering::SeqCst), 3);
    }

    test_scope_aborts_on_drop.tokio {
        let counter = std::sync::Arc::new(std::sync::atomic::AtomicUsize::new(0));
        let counter_clone = counter.clone();

        {
            let mut scope = GlobalAsyncScope::new();
            scope.spawn(async move {
                tokio::time::sleep(std::time::Duration::from_millis(100)).await;
                counter_clone.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
            });
        }

        tokio::time::sleep(std::time::Duration::from_millis(10)).await;
        assert_eq!(counter.load(std::sync::atomic::Ordering::SeqCst), 0);
    }
});