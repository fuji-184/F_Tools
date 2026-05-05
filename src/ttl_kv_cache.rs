use std::collections::HashMap;
use std::sync::RwLock;
use std::time::{Duration, Instant};

pub struct CacheItem<V> {
    value: V,
    expiry: Instant,
}

pub struct ThreadSafeTtlCache<K, V> {
    store: RwLock<HashMap<K, CacheItem<V>>>,
}

impl<K: std::hash::Hash + Eq, V: Clone> ThreadSafeTtlCache<K, V> {
    pub fn new() -> Self {
        Self { store: RwLock::new(HashMap::new()) }
    }

    pub fn insert(&self, key: K, value: V, ttl: Duration) {
        let mut store = self.store.write().unwrap();
        store.insert(key, CacheItem {
            value,
            expiry: Instant::now() + ttl,
        });
    }

    pub fn get(&self, key: &K) -> Option<V> {
        let store = self.store.read().unwrap();
        if let Some(item) = store.get(key) {
            if Instant::now() < item.expiry {
                return Some(item.value.clone());
            } else {
                drop(store);
                let mut store = self.store.write().unwrap();
                store.remove(key);
            }
        }
        None
    }
}