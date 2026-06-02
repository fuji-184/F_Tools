use futures::stream::FuturesUnordered;
use futures::StreamExt;

/*
Local structured concurrency scope for driving multiple concurrent operations to completion.

This utility groups related asynchronous tasks and guarantees that none of them outlive the 
enclosing scope block. It is primarily used to bundle and track dependent sub-operations—such 
as fanning out multiple web requests, parallelizing independent data parsing jobs, or executing 
batch background operations—ensuring that control only returns to the caller once every 
spawned task has successfully finished executing.
*/

use std::future::Future;

pub async fn local_async_scope<'a, F, T>(f: F) 
where 
    F: FnOnce(&mut LocalAsyncScope<'a>) -> T,
{
    let mut scope = LocalAsyncScope::default();
    f(&mut scope);
    
    while let Some(_) = scope.tasks.next().await {}
}

#[derive(Default)]
pub struct LocalAsyncScope<'a> {
    tasks: FuturesUnordered<std::pin::Pin<Box<dyn Future<Output = ()> + 'a + Send>>>,
}

impl<'a> LocalAsyncScope<'a> {
    pub fn spawn<Fut>(&mut self, f: Fut)
    where
        Fut: Future<Output = ()> + Send + 'a,
    {
        self.tasks.push(Box::pin(f));
    }
}

ftest::test!(local_async_scope_tests, {
    test_scope_spawns_and_awaits_all_tasks.tokio {
        let counter = std::sync::Arc::new(std::sync::atomic::AtomicUsize::new(0));
        let counter_clone1 = counter.clone();
        let counter_clone2 = counter.clone();

        local_async_scope(|scope| {
            scope.spawn(async move {
                tokio::time::sleep(std::time::Duration::from_millis(5)).await;
                counter_clone1.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
            });

            scope.spawn(async move {
                tokio::time::sleep(std::time::Duration::from_millis(10)).await;
                counter_clone2.fetch_add(2, std::sync::atomic::Ordering::SeqCst);
            });
        })
        .await;

        assert_eq!(counter.load(std::sync::atomic::Ordering::SeqCst), 3);
    }

    test_empty_scope_completes_immediately.tokio {
        let start = std::time::Instant::now();
        
        local_async_scope(|_scope| {}).await;

        assert!(start.elapsed() < std::time::Duration::from_millis(5));
    }
});