// Bloom
//
// HTTP REST API caching middleware
// Copyright: 2026, Valerian Saliou <valerian@valeriansaliou.name>
// License: Mozilla Public License v2.0 (MPL v2.0)

use std::collections::HashMap;
use std::sync::{Arc, Mutex};

use tokio::sync::{Mutex as AsyncMutex, OwnedMutexGuard};

lazy_static! {
    static ref PROXY_LOCK_MAP: Mutex<HashMap<String, Arc<AsyncMutex<()>>>> =
        Mutex::new(HashMap::new());
}

pub struct ProxyLock;

pub struct ProxyLockGuard {
    ns: String,
    mutex: Arc<AsyncMutex<()>>,
    guard: Option<OwnedMutexGuard<()>>,
}

impl ProxyLock {
    pub async fn acquire(ns: &str) -> (ProxyLockGuard, bool) {
        // Acquire a mutex
        let mutex = {
            PROXY_LOCK_MAP
                .lock()
                .expect("lock map poisoned")
                .entry(ns.to_string())
                .or_insert_with(|| Arc::new(AsyncMutex::new(())))
                .clone()
        };

        // Attempt to acquire the lock immediately; if it fails, wait for it.
        //   The boolean value indicates whether we had to wait or not \
        //   (ie. the lock was not free when we tried to obtain it), which the \
        //   caller uses to decide whether a cache re-check from Redis is \
        //   needed (since it might have been populated by the first lock \
        //   holder).
        let (guard, had_to_wait) = match Arc::clone(&mutex).try_lock_owned() {
            Ok(guard) => (guard, false),
            Err(_) => (Arc::clone(&mutex).lock_owned().await, true),
        };

        (
            ProxyLockGuard {
                ns: ns.to_string(),
                mutex,
                guard: Some(guard),
            },
            had_to_wait,
        )
    }
}

impl Drop for ProxyLockGuard {
    fn drop(&mut self) {
        // Release the async lock first so waiters can proceed to the \
        //   double-check
        drop(self.guard.take());

        // Remove the lock map entry only if it still points to our lock \
        //   reference
        let mut lock_map = PROXY_LOCK_MAP.lock().expect("lock map poisoned");

        if let Some(existing_lock) = lock_map.get(&self.ns) {
            if Arc::ptr_eq(existing_lock, &self.mutex) {
                lock_map.remove(&self.ns);
            }
        }
    }
}
