
/*
Thread-safe, Time-To-Live (TTL) based in-memory cache for expiring data.

This structure synchronizes concurrent read/write access to a hash map using an RwLock, 
ensuring that multiple threads can query the cache simultaneously while exclusive access 
is reserved for updates or eviction. It is primarily used to manage short-lived, frequently accessed data—such as authentication tokens, API response buffers, or ephemeral configuration 
flags—by automatically validating item freshness against a stored expiry timestamp and 
performing lazy eviction upon access to keep the memory footprint bounded.
*/

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

ftest::test!(thread_safe_ttl_cache_tests, {
    test_insert_and_get_valid {
        let cache = ThreadSafeTtlCache::new();
        cache.insert("key1", "value1".to_string(), Duration::from_secs(60));

        assert_eq!(cache.get(&"key1"), Some("value1".to_string()));
    }

    test_get_expired_returns_none_and_evicts {
        let cache = ThreadSafeTtlCache::new();
        cache.insert("key2", "value2".to_string(), Duration::from_nanos(1));

        std::thread::sleep(Duration::from_millis(1));

        assert_eq!(cache.get(&"key2"), None);
    }

    test_get_non_existent {
        let cache = ThreadSafeTtlCache::<i32, String>::new();
        assert_eq!(cache.get(&99), None);
    }
});