use std::cell::RefCell;
use std::collections::HashMap;
use std::hash::Hash;
use std::future::Future;
use std::pin::Pin;
use std::sync::Arc;

pub type AsyncBoxFut<U> = Pin<Box<dyn Future<Output = U> + 'static>>;

pub struct AsyncThreadLocalMemoize<T, U> {
    logic: Arc<dyn Fn(T) -> AsyncBoxFut<U> + 'static>,
    cache: Arc<RefCell<HashMap<T, U>>>,
    max_size: usize,
}

impl<T, U> AsyncThreadLocalMemoize<T, U>
where
    T: Hash + Eq + Clone + 'static,
    U: Clone + 'static,
{
    pub fn new<F, Fut>(max_size: usize, logic: F) -> Self
    where
        F: Fn(T) -> Fut + 'static,
        Fut: Future<Output = U> + 'static,
    {
        Self {
            logic: Arc::new(move |arg| Box::pin(logic(arg))),
            cache: Arc::new(RefCell::new(HashMap::with_capacity(max_size))),
            max_size,
        }
    }

    // Ubah return type menjadi penanda Future + 'static agar bisa keluar dari closure `.with()`
    pub fn call(&self, arg: T) -> impl Future<Output = U> + 'static {
        let logic = Arc::clone(&self.logic);
        let cache_clone = Arc::clone(&self.cache);
        let max_size = self.max_size;

        async move {
            {
                if let Some(result) = cache_clone.borrow().get(&arg) {
                    return result.clone();
                }
            }

            let result = (logic)(arg.clone()).await;

            let mut cache = cache_clone.borrow_mut();
            if cache.len() >= max_size {
                let key = cache.keys().next().cloned();
                if let Some(k) = key {
                    cache.remove(&k);
                }
            }
            cache.insert(arg, result.clone());
            result
        }
    }
}

#[macro_export]
macro_rules! async_thread_local_memo {
    ($name:ident, $max_size:expr, $logic:expr, $t:ty, $u:ty) => {
        use std::borrow::BorrowMut;
        thread_local! {
            static $name: $crate::memoize::AsyncThreadLocalMemoize<$t, $u> = 
                $crate::memoize::AsyncThreadLocalMemoize::new($max_size, $logic);
        }
    };
}

#[macro_export]
macro_rules! async_thread_local_memo2 {
    // name, size, logic, parameter type, return type
    ($name:ident, $max_size:expr, $logic:expr, $t:ty, $u:ty) => {
        use std::borrow::BorrowMut;
        thread_local! {
            static $name: $crate::memoize::AsyncThreadLocalMemoize<$t, $u, fn($t) -> $u> = 
                $crate::memoize::AsyncThreadLocalMemoize::new($max_size, $logic);
        }
    };
}


ftest::test!(async_thread_local_memoize_test, {
  
  test_memoize.tokio.skip {
    async_thread_local_memo!(M, 10, |(a, b): (i32, i32)| async move {
      a * b
    }, (i32, i32), i32);
    
    let a = [
          (2, 3, 6),
          (8, 2, 16),
          (9, 9, 81)
    ];
      
    for (a, b, c) in a {
        let res = M.with(|m| m.call((a, b))).await;
        assert_eq!(res, c);
    }
  }
  
});