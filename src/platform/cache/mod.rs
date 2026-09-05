pub mod lru;

use std::collections::HashMap;
use std::hash::{BuildHasher, Hash, RandomState};
use std::sync::RwLock;
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::{Duration, Instant};

type Shards<K, V> = Box<[RwLock<HashMap<K, Entry<V>>>]>;

struct Entry<V> {
    value: V,
    stored: Instant,
    touched: AtomicU64,
}

pub struct Cache<K, V> {
    shards: Shards<K, V>,
    hasher: RandomState,
    clock: AtomicU64,
    per_shard: usize,
    ttl: Option<Duration>,
}

impl<K: Eq + Hash + Clone, V: Clone> Cache<K, V> {
    pub fn new(capacity: usize, ttl: Option<Duration>) -> Self {
        Self {
            shards: (0..16)
                .map(|_| RwLock::new(HashMap::new()))
                .collect::<Vec<_>>()
                .into_boxed_slice(),
            hasher: RandomState::new(),
            clock: AtomicU64::new(0),
            per_shard: capacity.div_ceil(16).max(1),
            ttl,
        }
    }

    pub fn get(&self, key: &K) -> Option<V> {
        let shard = self.shards[self.shard_of(key)].read().ok()?;
        let entry = shard.get(key)?;

        if self.expired(entry) {
            return None;
        }

        entry.touched.store(
            self.clock.fetch_add(1, Ordering::Relaxed),
            Ordering::Relaxed,
        );

        Some(entry.value.clone())
    }

    pub fn insert(&self, key: K, value: V) {
        let Ok(mut shard) = self.shards[self.shard_of(&key)].write() else {
            return;
        };

        let stamp = self.clock.fetch_add(1, Ordering::Relaxed);

        if shard.len() >= self.per_shard
            && !shard.contains_key(&key)
            && let Some(doomed) = self.doomed(&shard)
        {
            shard.remove(&doomed);
        }

        shard.insert(
            key,
            Entry {
                value,
                stored: Instant::now(),
                touched: AtomicU64::new(stamp),
            },
        );
    }

    pub fn remove(&self, key: &K) -> Option<V> {
        self.shards[self.shard_of(key)]
            .write()
            .ok()?
            .remove(key)
            .map(|entry| entry.value)
    }

    pub fn sweep(&self) {
        for shard in &self.shards {
            let Ok(mut shard) = shard.write() else {
                continue;
            };

            shard.retain(|_, entry| !self.expired(entry));
        }
    }

    pub fn clear(&self) {
        for shard in &self.shards {
            if let Ok(mut shard) = shard.write() {
                shard.clear();
            }
        }
    }

    fn doomed(&self, shard: &HashMap<K, Entry<V>>) -> Option<K> {
        let mut coldest: Option<(&K, u64)> = None;

        for (key, entry) in shard {
            if self.expired(entry) {
                return Some(key.clone());
            }

            let touched = entry.touched.load(Ordering::Relaxed);

            if coldest.is_none_or(|(_, seen)| touched < seen) {
                coldest = Some((key, touched));
            }
        }

        coldest.map(|(key, _)| key.clone())
    }

    fn shard_of(&self, key: &K) -> usize {
        self.hasher.hash_one(key) as usize % 16
    }

    fn expired(&self, entry: &Entry<V>) -> bool {
        match self.ttl {
            Some(ttl) => entry.stored.elapsed() > ttl,
            None => false,
        }
    }
}

pub struct Debounce<K> {
    seen: Cache<K, Instant>,
    window: Duration,
}

impl<K: Eq + Hash + Clone> Debounce<K> {
    pub fn new(capacity: usize, window: Duration) -> Self {
        Self {
            seen: Cache::new(capacity, Some(window)),
            window,
        }
    }

    pub fn ready(&self, key: K) -> bool {
        if let Some(last) = self.seen.get(&key)
            && last.elapsed() < self.window
        {
            return false;
        }

        self.seen.insert(key, Instant::now());

        true
    }
}

pub struct TtlSet<K> {
    inner: Cache<K, ()>,
}

impl<K: Eq + Hash + Clone> TtlSet<K> {
    pub fn new(capacity: usize, ttl: Duration) -> Self {
        Self {
            inner: Cache::new(capacity, Some(ttl)),
        }
    }

    pub fn insert(&self, key: K) {
        self.inner.insert(key, ());
    }

    pub fn take(&self, key: &K) -> bool {
        self.inner.remove(key).is_some()
    }

    pub fn sweep(&self) {
        self.inner.sweep();
    }
}
