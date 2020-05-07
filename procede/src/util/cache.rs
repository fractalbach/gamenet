//! Module containing cache utilities for layers.
use std::cell::{RefCell, Ref};
use std::collections::hash_map::RandomState;
use std::marker::Send;
use std::hash::{Hash, BuildHasher};
use std::sync::Arc;

use cgmath::{Vector2, Vector3, Vector4};
use lru_cache::LruCache;
use thread_local::CachedThreadLocal;


// --------------------------------------------------------------------


/// Cache for a spherical surface.
///
/// Provides samples by bilinear interpolation between cached points.
pub struct BilinearSphereCache {
    cache: ShyInteriorCache<Vector4<i32>, BilinearSquare>,
}

#[derive(Clone)]
struct BilinearSquare {

}


/// "Shy" Interior cache.
///
/// This interior cache maintains an LRU cache using a RefCell, which
/// will be accessed only if it is not busy. If busy, the accessing
/// thread does not wait for it to become available, instead it just
/// uses the initializer function to re-produce the value.
///
/// This cache is best used where the initializer function does not
/// take significant time, or where the odds of two threads attempting
/// to access the cache simultaneously is small.
pub struct ShyInteriorCache<K: Eq + Hash + Clone, V: Clone> {
    cache: RefCell<LruCache<K, V>>
}

impl<K, V> ShyInteriorCache<K, V>
where K: Eq + Hash + Clone, V: Clone {
    pub fn new(cap: usize) -> ShyInteriorCache<K, V> {
        ShyInteriorCache { cache: RefCell::new(LruCache::new(cap)) }
    }

    /// Gets value, optionally using a copy of a cached value.
    ///
    /// If the cache is already busy, the initializer function will be
    /// called. This may occur if multiple threads are accessing the
    /// cache at the same time.
    pub fn get<F>(&self, key: &K, mut initializer: F) -> V
    where F: FnMut() -> V {
        {
            // Attempt to get cache.
            let mut cache = match self.cache.try_borrow_mut() {
                Ok(lru_cache) => lru_cache,
                Err(err) => return initializer()
            };

            // Attempt to get value from cache
            match cache.get_mut(key) {
                Some(value) => return value.clone(),
                None => {}
            }
        }

        // If no value was found in cache, initialize and add it.
        let value = initializer();
        match self.cache.try_borrow_mut() {
            Ok(mut lru_cache) => lru_cache.insert(key.clone(), value.clone()),
            Err(err) => None
        };
        value
    }
}

pub struct InteriorCache<K: Eq + Hash + Clone + Send, V: Send + Sync> {
    cap: usize,
    cache: CachedThreadLocal<RefCell<LruCache<K, Arc<V>>>>
}

impl<K, V> InteriorCache<K, V>
    where K: Eq + Hash + Clone + Send, V: Send + Sync {
    pub fn new(cap: usize) -> InteriorCache<K, V> {
        InteriorCache {
            cap,
            cache: CachedThreadLocal::new()
        }
    }

    /// Gets value, optionally using a copy of a cached value.
    ///
    /// If the cache is already busy, the initializer function will be
    /// called. This may occur if multiple threads are accessing the
    /// cache at the same time.
    pub fn get<F>(&self, k: &K, mut initializer: F) -> Arc<V>
        where F: FnMut() -> V {
        let cache_cell =
            self.cache.get_or(|| RefCell::new(LruCache::new(self.cap)));
        let mut cache = cache_cell.borrow_mut();
        if !cache.contains_key(k) {
            cache.insert(k.clone(), Arc::new(initializer()));
        }
        cache.get_mut(k).unwrap().clone()
    }
}

#[cfg(test)]
mod tests {
    use ::util::cache::*;

    #[test]
    fn test_cache_initializes_value_once() {
        let mut count = 0;

        let cache = ShyInteriorCache::new(10);

        let b = cache.get(&2, ||{2});
        for _ in 0..50 {
            let a = cache.get(&1, || {
                count += 1;
                1
            });
            assert_eq!(a, 1);
        }

        assert_eq!(count, 1);
    }
}
