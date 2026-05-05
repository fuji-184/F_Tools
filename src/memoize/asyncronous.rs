use std::collections::HashMap;
use std::hash::Hash;
use std::future::Future;
use std::sync::Arc;

#[cfg(feature = "async_cross_thread")]
use tokio::sync::Mutex;


#[cfg(feature = "async_cross_thread")]
pub struct AsyncMemoize<T, U, F> {
    logic: F,
    cache: Arc<Mutex<HashMap<T, U>>>,
    max_size: usize,
}


#[cfg(feature = "async_cross_thread")]
impl<T, U, F, Fut> AsyncMemoize<T, U, F>
where
    T: Hash + Eq + Clone + Send + 'static,
    U: Clone + Send + 'static,
    F: Fn(T) -> Fut + Send + Sync + 'static,
    Fut: Future<Output = U> + Send,
{
    pub fn new(max_size: usize, logic: F) -> Self {
        AsyncMemoize {
            logic,
            cache: Arc::new(Mutex::new(HashMap::with_capacity(max_size))),
            max_size,
        }
    }

    pub fn call(&self, arg: T) -> impl Future<Output = U> + '_ {
        async move {
            let mut cache = self.cache.lock().await;
            
            if let Some(result) = cache.get(&arg) {
                return result.clone();
            }

            let result = (self.logic)(arg.clone()).await;

            if cache.len() >= self.max_size {
                let key_to_remove = cache.keys().next().cloned();
                if let Some(k) = key_to_remove {
                    cache.remove(&k);
                }
            }

            cache.insert(arg, result.clone());
            result
        }
    }
}
