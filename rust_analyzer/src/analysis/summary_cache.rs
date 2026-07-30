//! Summary cache for cross-function taint analysis.
//!
//! Each function's taint behavior is pre-computed and cached.
//! When analyzing callers, the cache avoids re-entering the callee.
//! Cache is invalidated when the function source changes (based on git diff).

use super::FunctionTaintSummary;
use dashmap::DashMap;
use std::time::{Duration, Instant};

struct CacheEntry {
    summary: FunctionTaintSummary,
    inserted_at: Instant,
    ttl: Duration,
}

pub struct SummaryCache {
    cache: DashMap<String, CacheEntry>,
}

impl SummaryCache {
    pub fn new() -> Self {
        Self {
            cache: DashMap::new(),
        }
    }

    /// Get a cached summary if still valid
    pub fn get(&self, function_name: &str) -> Option<FunctionTaintSummary> {
        self.cache.get(function_name).and_then(|entry| {
            if entry.inserted_at.elapsed() < entry.ttl {
                Some(entry.summary.clone())
            } else {
                // TTL expired, remove stale entry
                drop(entry);
                self.cache.remove(function_name);
                None
            }
        })
    }

    /// Insert or update a summary
    pub fn insert(&self, function_name: String, summary: FunctionTaintSummary) {
        self.cache.insert(function_name, CacheEntry {
            summary,
            inserted_at: Instant::now(),
            ttl: Duration::from_secs(3600), // Default 1 hour
        });
    }

    /// Invalidate cache entries for functions changed in the given files
    pub fn invalidate_changed(&self, changed_functions: &[String]) {
        for func in changed_functions {
            self.cache.remove(func);
        }
    }

    /// Invalidate functions whose source hash changed
    pub fn invalidate_by_hash(&self, function_name: &str, new_hash: &str) -> bool {
        if let Some(entry) = self.cache.get(function_name) {
            if entry.summary.hash != new_hash {
                drop(entry);
                self.cache.remove(function_name);
                return true;
            }
        }
        false
    }

    pub fn len(&self) -> usize {
        self.cache.len()
    }

    pub fn is_empty(&self) -> bool {
        self.cache.is_empty()
    }
}

impl Default for SummaryCache {
    fn default() -> Self {
        Self::new()
    }
}
