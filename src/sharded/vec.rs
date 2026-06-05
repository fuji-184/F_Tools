
// Thread-safe Vec partitioned across multiple RwLock-protected shards, where the caller
// explicitly controls which shard an item lands on via a shard_key rather than deriving
// it from the item's hash. This makes ShardedVec suitable for workloads where grouping
// is meaningful — e.g. partitioning work items by thread id, task category, or bucket index.
// push() and pop() acquire an exclusive write lock on the target shard only.
// get() acquires a shared read lock, allowing concurrent reads across the same shard.
// len_of_shard() reports the element count of a single shard, not the total across all shards.
// shard_key is reduced via modulo, so any usize value is a valid key.
use std::sync::RwLock;

pub struct ShardedVec<T> {
    shards: Vec<RwLock<Vec<T>>>,
    num_shards: usize,
}

impl<T: Clone> ShardedVec<T> {
    pub fn new(num_shards: usize) -> Self {
        let mut shards = Vec::with_capacity(num_shards);
        for _ in 0..num_shards {
            shards.push(RwLock::new(Vec::new()));
        }
        Self { shards, num_shards }
    }

    pub fn push(&self, item: T, shard_key: usize) {
        let idx = shard_key % self.num_shards;
        let mut shard = self.shards[idx].write().unwrap();
        shard.push(item);
    }

    pub fn get(&self, shard_key: usize, index: usize) -> Option<T> {
        let idx = shard_key % self.num_shards;
        let shard = self.shards[idx].read().unwrap();
        shard.get(index).cloned()
    }

    pub fn pop(&self, shard_key: usize) -> Option<T> {
        let idx = shard_key % self.num_shards;
        let mut shard = self.shards[idx].write().unwrap();
        shard.pop()
    }

    pub fn len_of_shard(&self, shard_key: usize) -> usize {
        let idx = shard_key % self.num_shards;
        let shard = self.shards[idx].read().unwrap();
        shard.len()
    }
}

ftest::test!(sharded_vec_tests, {

    push_and_get {
        let vec = ShardedVec::new(4);
        vec.push(42i32, 0);
        assert_eq!(vec.get(0, 0), Some(42));
    }

    get_out_of_bounds_returns_none {
        let vec = ShardedVec::<i32>::new(4);
        assert_eq!(vec.get(0, 99), None);
    }

    pop_returns_last_pushed {
        let vec = ShardedVec::new(4);
        vec.push(1i32, 0);
        vec.push(2i32, 0);
        assert_eq!(vec.pop(0), Some(2));
    }

    pop_empty_shard_returns_none {
        let vec = ShardedVec::<i32>::new(4);
        assert_eq!(vec.pop(0), None);
    }

    len_of_shard_counts_correctly {
        let vec = ShardedVec::new(4);
        vec.push(1i32, 0);
        vec.push(2i32, 0);
        vec.push(3i32, 0);
        assert_eq!(vec.len_of_shard(0), 3);
    }

    shard_key_modulo_wraps_correctly {
        let vec = ShardedVec::new(4);
        vec.push(10i32, 0);
        vec.push(20i32, 4);
        assert_eq!(vec.len_of_shard(0), 2);
    }

    different_shard_keys_are_isolated {
        let vec = ShardedVec::new(4);
        vec.push(1i32, 0);
        vec.push(2i32, 1);
        assert_eq!(vec.len_of_shard(0), 1);
        assert_eq!(vec.len_of_shard(1), 1);
        assert_eq!(vec.get(0, 0), Some(1));
        assert_eq!(vec.get(1, 0), Some(2));
    }

    single_shard_still_works {
        let vec = ShardedVec::new(1);
        vec.push(10i32, 0);
        vec.push(20i32, 5);
        assert_eq!(vec.len_of_shard(0), 2);
    }

    concurrent_pushes_to_different_shards {
        use std::sync::Arc;
        let vec = Arc::new(ShardedVec::new(8));
        let mut handles = vec![];
        for i in 0..8usize {
            let v = Arc::clone(&vec);
            handles.push(std::thread::spawn(move || {
                v.push(i as i32, i);
            }));
        }
        for h in handles { h.join().unwrap(); }
        for i in 0..8usize {
            assert_eq!(vec.len_of_shard(i), 1);
        }
    }
});