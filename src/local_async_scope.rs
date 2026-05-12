use futures::stream::FuturesUnordered;
use futures::StreamExt;
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