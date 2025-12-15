use std::hash::{Hash, Hasher};
use std::time::Duration;

use moka::sync::Cache;

use blueprint_common::{OpId, RecordedValue, ValueRef};

#[derive(Debug, Clone)]
pub struct CachedResult {
    pub value: RecordedValue,
    pub input_hash: u64,
}

impl CachedResult {
    pub fn new(value: RecordedValue, input_hash: u64) -> Self {
        Self { value, input_hash }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CachePolicy {
    Normal,
    NoCache,
    ForceRefresh,
}

impl Default for CachePolicy {
    fn default() -> Self {
        Self::Normal
    }
}

const DEFAULT_MAX_CAPACITY: u64 = 10_000;
const DEFAULT_TTL_SECS: u64 = 3600;

#[derive(Clone)]
pub struct OpCache {
    cache: Cache<(OpId, u64), RecordedValue>,
    value_cache: Cache<OpId, RecordedValue>,
}

impl OpCache {
    pub fn new() -> Self {
        Self::with_config(DEFAULT_MAX_CAPACITY, Duration::from_secs(DEFAULT_TTL_SECS))
    }

    pub fn with_ttl(ttl: Duration) -> Self {
        Self::with_config(DEFAULT_MAX_CAPACITY, ttl)
    }

    pub fn with_config(max_capacity: u64, ttl: Duration) -> Self {
        Self {
            cache: Cache::builder()
                .max_capacity(max_capacity)
                .time_to_live(ttl)
                .build(),
            value_cache: Cache::builder()
                .max_capacity(max_capacity)
                .time_to_live(ttl)
                .build(),
        }
    }

    pub fn get(&self, op_id: OpId, input_hash: u64) -> Option<CachedResult> {
        self.cache.get(&(op_id, input_hash)).map(|value| {
            CachedResult { value, input_hash }
        })
    }

    pub fn get_value(&self, op_id: OpId) -> Option<RecordedValue> {
        self.value_cache.get(&op_id)
    }

    pub fn insert(&self, op_id: OpId, value: RecordedValue, input_hash: u64) {
        self.cache.insert((op_id, input_hash), value.clone());
        self.value_cache.insert(op_id, value);
    }

    pub fn invalidate(&self, op_id: OpId) {
        self.value_cache.invalidate(&op_id);
    }

    pub fn clear(&self) {
        self.cache.invalidate_all();
        self.value_cache.invalidate_all();
    }

    pub fn len(&self) -> usize {
        self.value_cache.entry_count() as usize
    }

    pub fn is_empty(&self) -> bool {
        self.value_cache.entry_count() == 0
    }

    pub fn sync(&self) {
        self.cache.run_pending_tasks();
        self.value_cache.run_pending_tasks();
    }
}

impl Default for OpCache {
    fn default() -> Self {
        Self::new()
    }
}

impl std::fmt::Debug for OpCache {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("OpCache")
            .field("len", &self.len())
            .finish()
    }
}

pub fn compute_input_hash(inputs: &[ValueRef]) -> u64 {
    use std::collections::hash_map::DefaultHasher;
    let mut hasher = DefaultHasher::new();
    for input in inputs {
        hash_value_ref(input, &mut hasher);
    }
    hasher.finish()
}

fn hash_value_ref<H: Hasher>(value_ref: &ValueRef, hasher: &mut H) {
    match value_ref {
        ValueRef::Literal(v) => {
            0u8.hash(hasher);
            hash_recorded_value(v, hasher);
        }
        ValueRef::OpOutput { op, path } => {
            1u8.hash(hasher);
            op.hash(hasher);
            for accessor in path {
                accessor.hash(hasher);
            }
        }
        ValueRef::Dynamic(s) => {
            2u8.hash(hasher);
            s.hash(hasher);
        }
        ValueRef::List(items) => {
            3u8.hash(hasher);
            items.len().hash(hasher);
            for item in items {
                hash_value_ref(item, hasher);
            }
        }
    }
}

fn hash_recorded_value<H: Hasher>(value: &RecordedValue, hasher: &mut H) {
    match value {
        RecordedValue::None => 0u8.hash(hasher),
        RecordedValue::Bool(b) => {
            1u8.hash(hasher);
            b.hash(hasher);
        }
        RecordedValue::Int(i) => {
            2u8.hash(hasher);
            i.hash(hasher);
        }
        RecordedValue::Float(f) => {
            3u8.hash(hasher);
            f.to_bits().hash(hasher);
        }
        RecordedValue::String(s) => {
            4u8.hash(hasher);
            s.hash(hasher);
        }
        RecordedValue::Bytes(b) => {
            5u8.hash(hasher);
            b.hash(hasher);
        }
        RecordedValue::List(list) => {
            6u8.hash(hasher);
            for item in list {
                hash_recorded_value(item, hasher);
            }
        }
        RecordedValue::Dict(dict) => {
            7u8.hash(hasher);
            for (k, v) in dict {
                k.hash(hasher);
                hash_recorded_value(v, hasher);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cache_insert_and_get() {
        let cache = OpCache::new();
        let op_id = OpId(0);
        let value = RecordedValue::String("test".to_string());
        let hash = 12345u64;

        cache.insert(op_id, value.clone(), hash);
        cache.sync();

        let cached = cache.get(op_id, hash);
        assert!(cached.is_some());
        assert_eq!(cached.unwrap().input_hash, hash);
    }

    #[test]
    fn test_cache_miss_wrong_hash() {
        let cache = OpCache::new();
        let op_id = OpId(0);
        let value = RecordedValue::String("test".to_string());

        cache.insert(op_id, value, 12345);
        cache.sync();

        let cached = cache.get(op_id, 99999);
        assert!(cached.is_none());
    }

    #[test]
    fn test_cache_invalidate() {
        let cache = OpCache::new();
        let op_id = OpId(0);
        let value = RecordedValue::String("test".to_string());

        cache.insert(op_id, value, 12345);
        cache.sync();
        assert!(!cache.is_empty());

        cache.invalidate(op_id);
        cache.sync();
        assert!(cache.is_empty());
    }

    #[test]
    fn test_get_value() {
        let cache = OpCache::new();
        let op_id = OpId(0);
        let value = RecordedValue::Int(42);

        cache.insert(op_id, value.clone(), 12345);
        cache.sync();

        let retrieved = cache.get_value(op_id);
        assert_eq!(retrieved, Some(value));
    }

    #[test]
    fn test_clear() {
        let cache = OpCache::new();
        cache.insert(OpId(0), RecordedValue::Int(1), 1);
        cache.insert(OpId(1), RecordedValue::Int(2), 2);
        cache.sync();

        assert_eq!(cache.len(), 2);

        cache.clear();
        cache.sync();

        assert!(cache.is_empty());
    }
}
