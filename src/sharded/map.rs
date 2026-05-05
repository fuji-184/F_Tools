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