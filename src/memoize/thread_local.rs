use std::cell::RefCell;
use std::collections::HashMap;
use std::hash::Hash;

pub struct ThreadLocalMemoize<T, U, F> {
    logic: F,
    cache: HashMap<T, U>,
    max_size: usize,
}

impl<T, U, F> ThreadLocalMemoize<T, U, F>
where
    T: Hash + Eq + Clone,
    U: Clone,
    F: FnMut(T) -> U,
{
    pub fn new(max_size: usize, logic: F) -> Self {
        Self {
            logic,
            cache: HashMap::with_capacity(max_size),
            max_size,
        }
    }

    pub fn call(&mut self, arg: T) -> U {
        if let Some(result) = self.cache.get(&arg) {
            return result.clone();
        }

        let result = (self.logic)(arg.clone());

        if self.cache.len() >= self.max_size {
            let key = self.cache.keys().next().cloned();
            if let Some(k) = key {
                self.cache.remove(&k);
            }
        }

        self.cache.insert(arg, result.clone());
        result
    }
}

#[macro_export]
macro_rules! thread_local_memo {
    ($name:ident, $max_size:expr, $logic:expr, $t:ty, $u:ty) => {
        thread_local! {
            static $name: std::cell::RefCell<$crate::memoize::ThreadLocalMemoize<$t, $u, fn($t) -> $u>> = 
                std::cell::RefCell::new($crate::memoize::ThreadLocalMemoize::new($max_size, $logic));
        }
    };
}
