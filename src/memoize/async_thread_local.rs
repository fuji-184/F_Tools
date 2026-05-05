use std::cell::RefCell;
use std::collections::HashMap;
use std::hash::Hash;
use std::future::Future;

pub struct AsyncThreadLocalMemoize<T, U, F> {
    logic: F,
    cache: RefCell<HashMap<T, U>>,
    max_size: usize,
}

impl<T, U, F, Fut> AsyncThreadLocalMemoize<T, U, F>
where
    T: Hash + Eq + Clone + 'static,
    U: Clone + 'static,
    F: Fn(T) -> Fut + 'static,
    Fut: Future<Output = U>,
{
    pub fn new(max_size: usize, logic: F) -> Self {
        Self {
            logic,
            cache: RefCell::new(HashMap::with_capacity(max_size)),
            max_size,
        }
    }

    pub async fn call(&self, arg: T) -> U {
        if let Some(result) = self.cache.borrow().get(&arg) {
            return result.clone();
        }

        let result = (self.logic)(arg.clone()).await;

        let mut cache = self.cache.borrow_mut();
        if cache.len() >= self.max_size {
            let key = cache.keys().next().cloned();
            if let Some(k) = key {
                cache.remove(&k);
            }
        }
        cache.insert(arg, result.clone());
        result
    }
}

#[macro_export]
macro_rules! async_thread_local_memo {
    ($name:ident, $max_size:expr, $logic:expr, $t:ty, $u:ty) => {
        use std::borrow::BorrowMut;
        thread_local! {
            static $name: $crate::memoize::AsyncThreadLocalMemoize<$t, $u, fn($t) -> $u> = 
                $crate::memoize::AsyncThreadLocalMemoize::new($max_size, $logic);
        }
    };
}