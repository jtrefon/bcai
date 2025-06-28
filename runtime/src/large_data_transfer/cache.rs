//! Cache Implementation for Large Data Transfer
//!
//! Provides a simple in-memory LRU cache used by the large data transfer
//! system.  This is intentionally lightweight but functional so higher level
//! components can store and retrieve chunks without hitting disk each time.

use std::collections::{HashMap, VecDeque};

/// Basic LRU cache for chunks identified by a string key.
#[derive(Debug)]
pub struct ChunkCache {
    max_size: u64,
    current_size: u64,
    map: HashMap<String, Vec<u8>>,
    order: VecDeque<String>,
}

impl ChunkCache {
    /// Create a new cache with the given capacity in bytes.
    pub fn new(max_size: u64) -> Self {
        Self {
            max_size,
            current_size: 0,
            map: HashMap::new(),
            order: VecDeque::new(),
        }
    }

    /// Insert or update a cached entry. Old entries are evicted if capacity
    /// would be exceeded. Oversized items are dropped.
    pub fn put(&mut self, key: String, data: Vec<u8>) {
        let size = data.len() as u64;
        if size > self.max_size {
            return;
        }

        if let Some(old) = self.map.remove(&key) {
            self.current_size -= old.len() as u64;
            self.order.retain(|k| k != &key);
        }

        while self.current_size + size > self.max_size {
            if let Some(old_key) = self.order.pop_front() {
                if let Some(old_data) = self.map.remove(&old_key) {
                    self.current_size -= old_data.len() as u64;
                }
            } else {
                break;
            }
        }

        self.current_size += size;
        self.map.insert(key.clone(), data);
        self.order.push_back(key);
    }

    /// Retrieve a value from the cache. Marks the entry as recently used.
    pub fn get(&mut self, key: &str) -> Option<&[u8]> {
        if let Some(data) = self.map.get(key) {
            self.order.retain(|k| k != key);
            self.order.push_back(key.to_string());
            Some(data)
        } else {
            None
        }
    }

    /// Remove an entry from the cache.
    pub fn remove(&mut self, key: &str) -> Option<Vec<u8>> {
        if let Some(data) = self.map.remove(key) {
            self.order.retain(|k| k != key);
            self.current_size -= data.len() as u64;
            Some(data)
        } else {
            None
        }
    }
}
