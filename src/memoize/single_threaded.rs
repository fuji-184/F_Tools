use std::collections::HashMap;
use std::hash::Hash;

pub struct MemoizeSingleThreaded<T, U, F>
where
    T: Hash + Eq,
    U: Clone,
    F: FnMut(&T) -> U,
{
    logic: F,
    cache: HashMap<T, U>,
    max_size: usize,
}

impl<T, U, F> MemoizeSingleThreaded<T, U, F>
where
    T: Hash + Eq + Clone,
    U: Clone,
    F: FnMut(&T) -> U,
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

        let result = (self.logic)(&arg);

        if self.cache.len() >= self.max_size {
            let key_to_remove = self.cache.keys().next().cloned();
            if let Some(k) = key_to_remove {
                self.cache.remove(&k);
            }
        }

        self.cache.insert(arg, result.clone());
        result
    }
}

ftest::test!(memoize_single_thrraded_test, {

    test_memoize {
      let mut m = MemoizeSingleThreaded::new(10, |&(a, b): &(i32, i32)| {
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