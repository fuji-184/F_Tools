
// Thread-safe HashSet that partitions entries across multiple RwLock-protected shards
// to reduce lock contention under concurrent workloads. Mirrors the design of ShardedMap
// but stores only keys with no associated values. Each operation hashes the item to
// determine which shard to lock, so concurrent operations on different items rarely
// contend on the same lock. contains() and len() acquire shared read locks, while
// insert() and remove() acquire exclusive write locks only on the affected shard.
// len() acquires a read lock on every shard sequentially and sums the counts,
// so it is not atomic — the result may be stale under concurrent modification.
use std::sync::RwLock;
use std::collections::HashSet;
use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};

pub struct ShardedSet<T> {
    shards: Vec<RwLock<HashSet<T>>>,
    num_shards: usize,
}

impl<T: Hash + Eq + Clone> ShardedSet<T> {
    pub fn new(num_shards: usize) -> Self {
        let mut shards = Vec::with_capacity(num_shards);
        for _ in 0..num_shards {
            shards.push(RwLock::new(HashSet::new()));
        }
        Self { shards, num_shards }
    }

    fn get_shard_idx(&self, item: &T) -> usize {
        let mut s = DefaultHasher::new();
        item.hash(&mut s);
        (s.finish() as usize) % self.num_shards
    }

    pub fn insert(&self, item: T) {
        let idx = self.get_shard_idx(&item);
        let mut shard = self.shards[idx].write().unwrap();
        shard.insert(item);
    }

    pub fn contains(&self, item: &T) -> bool {
        let idx = self.get_shard_idx(item);
        let shard = self.shards[idx].read().unwrap();
        shard.contains(item)
    }

    pub fn remove(&self, item: &T) -> bool {
        let idx = self.get_shard_idx(item);
        let mut shard = self.shards[idx].write().unwrap();
        shard.remove(item)
    }

    pub fn len(&self) -> usize {
        self.shards.iter()
            .map(|s| s.read().unwrap().len())
            .sum()
        }
}

ftest::test!(sharded_set_tests, {

    insert_and_contains {
        let set = ShardedSet::new(4);
        set.insert("hello");
        assert!(set.contains(&"hello"));
    }

    contains_missing_returns_false {
        let set = ShardedSet::<&str>::new(4);
        assert!(!set.contains(&"missing"));
    }

    insert_duplicate_no_growth {
        let set = ShardedSet::new(4);
        set.insert("dup");
        set.insert("dup");
        assert_eq!(set.len(), 1);
    }

    remove_existing_returns_true {
        let set = ShardedSet::new(4);
        set.insert("item");
        assert!(set.remove(&"item"));
        assert!(!set.contains(&"item"));
    }

    remove_missing_returns_false {
        let set = ShardedSet::<&str>::new(4);
        assert!(!set.remove(&"ghost"));
    }

    len_counts_across_shards {
        let set = ShardedSet::new(4);
        for i in 0..10i32 {
            set.insert(i);
        }
        assert_eq!(set.len(), 10);
    }

    len_empty_set {
        let set = ShardedSet::<i32>::new(4);
        assert_eq!(set.len(), 0);
    }

    single_shard_still_works {
        let set = ShardedSet::new(1);
        set.insert(1i32);
        set.insert(2i32);
        assert!(set.contains(&1));
        assert!(set.contains(&2));
        assert_eq!(set.len(), 2);
    }

    concurrent_inserts_and_contains {
        use std::sync::Arc;
        let set = Arc::new(ShardedSet::new(8));
        let mut handles = vec![];
        for i in 0..8i32 {
            let s = Arc::clone(&set);
            handles.push(std::thread::spawn(move || {
                s.insert(i);
            }));
        }
        for h in handles { h.join().unwrap(); }
        for i in 0..8i32 {
            assert!(set.contains(&i));
        }
        assert_eq!(set.len(), 8);
    }
});