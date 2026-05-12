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

        futures::executor::block_on(async {
            for handle in self.handles.drain(..) {
                let _ = handle.await;
            }
        });
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