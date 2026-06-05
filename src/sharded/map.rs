
// Thread-safe HashMap that partitions entries across multiple RwLock-protected shards
// to reduce lock contention under concurrent workloads. Keys are distributed across
// shards using DefaultHasher, ensuring each operation only locks one shard at a time
// rather than a single global lock. Read operations (get, contains_key) acquire a
// shared read lock allowing multiple concurrent readers per shard, while write
// operations (insert, remove) acquire an exclusive write lock only on the affected shard.
// num_shards controls the concurrency granularity — higher shard counts reduce contention
// at the cost of memory overhead from additional RwLock allocations.
use std::sync::RwLock;
use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};

pub struct ShardedMap<K, V> {
    shards: Vec<RwLock<std::collections::HashMap<K, V>>>,
    num_shards: usize,
}

impl<K: Hash + Eq + Clone, V: Clone> ShardedMap<K, V> {
    pub fn new(num_shards: usize) -> Self {
        let mut shards = Vec::with_capacity(num_shards);
        for _ in 0..num_shards {
            shards.push(RwLock::new(std::collections::HashMap::new()));
        }
        Self { shards, num_shards }
    }

    fn get_shard_idx(&self, key: &K) -> usize {
        let mut s = DefaultHasher::new();
        key.hash(&mut s);
        (s.finish() as usize) % self.num_shards
    }

    pub fn insert(&self, key: K, value: V) {
        let idx = self.get_shard_idx(&key);
        let mut shard = self.shards[idx].write().unwrap();
        shard.insert(key, value);
    }

    pub fn get(&self, key: &K) -> Option<V> {
        let idx = self.get_shard_idx(key);
        let shard = self.shards[idx].read().unwrap();
        shard.get(key).cloned()
    }

    pub fn remove(&self, key: &K) -> Option<V> {
        let idx = self.get_shard_idx(key);
        let mut shard = self.shards[idx].write().unwrap();
        shard.remove(key)
    }

    pub fn contains_key(&self, key: &K) -> bool {
        let idx = self.get_shard_idx(key);
        let shard = self.shards[idx].read().unwrap();
        shard.contains_key(key)
    }
}

ftest::test!(sharded_map_tests, {

    insert_and_get {
        let map = ShardedMap::new(4);
        map.insert("key", 42);
        assert_eq!(map.get(&"key"), Some(42));
    }

    get_missing_key_returns_none {
        let map = ShardedMap::<&str, i32>::new(4);
        assert_eq!(map.get(&"missing"), None);
    }

    insert_overwrites_existing {
        let map = ShardedMap::new(4);
        map.insert("key", 1);
        map.insert("key", 2);
        assert_eq!(map.get(&"key"), Some(2));
    }

    remove_returns_value {
        let map = ShardedMap::new(4);
        map.insert("key", 99);
        assert_eq!(map.remove(&"key"), Some(99));
        assert_eq!(map.get(&"key"), None);
    }

    remove_missing_returns_none {
        let map = ShardedMap::<&str, i32>::new(4);
        assert_eq!(map.remove(&"ghost"), None);
    }

    contains_key_true {
        let map = ShardedMap::new(4);
        map.insert("present", 1);
        assert!(map.contains_key(&"present"));
    }

    contains_key_false {
        let map = ShardedMap::<&str, i32>::new(4);
        assert!(!map.contains_key(&"absent"));
    }

    multiple_keys_across_shards {
        let map = ShardedMap::new(4);
        for i in 0..20i32 {
            map.insert(i, i * 10);
        }
        for i in 0..20i32 {
            assert_eq!(map.get(&i), Some(i * 10));
        }
    }

    single_shard_still_works {
        let map = ShardedMap::new(1);
        map.insert("a", 1);
        map.insert("b", 2);
        assert_eq!(map.get(&"a"), Some(1));
        assert_eq!(map.get(&"b"), Some(2));
    }

    concurrent_inserts_and_reads {
        use std::sync::Arc;
        let map = Arc::new(ShardedMap::new(8));
        let mut handles = vec![];
        for i in 0..8 {
            let m = Arc::clone(&map);
            handles.push(std::thread::spawn(move || {
                m.insert(i, i * 100);
            }));
        }
        for h in handles { h.join().unwrap(); }
        for i in 0..8 {
            assert_eq!(map.get(&i), Some(i * 100));
        }
    }
});