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

ftest::test!(memoize_thread_safe_test, {

    test_memoize {
      let m = MemoizeThreadSafe::new(10, |(a, b): (i32, i32)| {
        a * b
      });
      
      let a = [
          (2, 3, 6),
          (8, 2, 16),
          (9, 9, 81)
      ];
      
      for (a, b, c) in a {
          let res = m.call((a, b));
          assert_eq!(res, c);
      }
    }
    
});