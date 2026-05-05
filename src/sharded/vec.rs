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