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