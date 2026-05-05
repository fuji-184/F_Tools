use std::collections::HashMap;
use std::hash::Hash;
use std::sync::RwLock;

pub struct MemoizeThreadSafe<T, U, F> {
    logic: F,
    cache: RwLock<HashMap<T, U>>,
    max_size: usize,
}

impl<T, U, F> MemoizeThreadSafe<T, U, F>
where
    T: Hash + Eq + Clone + Send + Sync,
    U: Clone + Send + Sync,
    F: Fn(T) -> U + Send + Sync,
{
    pub fn new(max_size: usize, logic: F) -> Self {
        Self {
            logic,
            cache: RwLock::new(HashMap::with_capacity(max_size)),
            max_size,
        }
    }

    pub fn call(&self, arg: T) -> U {
        {
            let read_guard = self.cache.read().unwrap();
            if let Some(result) = read_guard.get(&arg) {
                return result.clone();
            }
        }

        let result = (self.logic)(arg.clone());

        let mut write_guard = self.cache.write().unwrap();
        
        if write_guard.len() >= self.max_size {
            let key_to_remove = write_guard.keys().next().cloned();
            if let Some(k) = key_to_remove {
                write_guard.remove(&k);
            }
        }

        write_guard.insert(arg, result.clone());
        result
    }
}
