use std::collections::HashMap;
use blueprint_common::{Accessor, OpId, RecordedValue, ValueRef};
use super::cache::OpCache;

pub struct ValueResolver<'a> {
    cache: &'a OpCache,
    params: Option<&'a HashMap<String, RecordedValue>>,
    local_results: Option<&'a HashMap<OpId, RecordedValue>>,
}

impl<'a> ValueResolver<'a> {
    pub fn new(cache: &'a OpCache) -> Self {
        Self { cache, params: None, local_results: None }
    }

    pub fn with_params(mut self, params: &'a HashMap<String, RecordedValue>) -> Self {
        self.params = Some(params);
        self
    }

    pub fn with_local_results(mut self, local_results: &'a HashMap<OpId, RecordedValue>) -> Self {
        self.local_results = Some(local_results);
        self
    }

    pub fn resolve(&self, value_ref: &ValueRef) -> Option<RecordedValue> {
        match value_ref {
            ValueRef::Literal(value) => Some(value.clone()),
            ValueRef::OpOutput { op, path } => {
                let base_value = self.local_results
                    .and_then(|lr| lr.get(op).cloned())
                    .or_else(|| self.cache.get_value(*op))?;
                self.resolve_path(&base_value, path)
            }
            ValueRef::Dynamic(name) => {
                self.params.and_then(|p| p.get(name).cloned())
            }
            ValueRef::List(items) => {
                let resolved: Option<Vec<RecordedValue>> = items.iter()
                    .map(|item| self.resolve(item))
                    .collect();
                resolved.map(RecordedValue::List)
            }
        }
    }

    pub fn resolve_to_string(&self, value_ref: &ValueRef) -> Option<String> {
        match self.resolve(value_ref)? {
            RecordedValue::String(s) => Some(s),
            RecordedValue::Int(i) => Some(i.to_string()),
            RecordedValue::Float(f) => Some(f.to_string()),
            RecordedValue::Bool(b) => Some(b.to_string()),
            RecordedValue::None => Some("None".to_string()),
            _ => None,
        }
    }

    pub fn resolve_to_int(&self, value_ref: &ValueRef) -> Option<i64> {
        match self.resolve(value_ref)? {
            RecordedValue::Int(i) => Some(i),
            RecordedValue::String(s) => s.parse().ok(),
            _ => None,
        }
    }

    pub fn resolve_to_float(&self, value_ref: &ValueRef) -> Option<f64> {
        match self.resolve(value_ref)? {
            RecordedValue::Float(f) => Some(f),
            RecordedValue::Int(i) => Some(i as f64),
            RecordedValue::String(s) => s.parse().ok(),
            _ => None,
        }
    }

    pub fn resolve_to_bool(&self, value_ref: &ValueRef) -> Option<bool> {
        match self.resolve(value_ref)? {
            RecordedValue::Bool(b) => Some(b),
            RecordedValue::String(s) => match s.as_str() {
                "true" | "True" | "1" => Some(true),
                "false" | "False" | "0" => Some(false),
                _ => None,
            },
            RecordedValue::Int(i) => Some(i != 0),
            _ => None,
        }
    }

    pub fn resolve_to_bytes(&self, value_ref: &ValueRef) -> Option<Vec<u8>> {
        match self.resolve(value_ref)? {
            RecordedValue::Bytes(b) => Some(b),
            RecordedValue::String(s) => Some(s.into_bytes()),
            _ => None,
        }
    }

    pub fn resolve_to_list(&self, value_ref: &ValueRef) -> Option<Vec<RecordedValue>> {
        match self.resolve(value_ref)? {
            RecordedValue::List(l) => Some(l),
            _ => None,
        }
    }

    fn resolve_path(&self, base: &RecordedValue, path: &[Accessor]) -> Option<RecordedValue> {
        let mut current = base.clone();

        for accessor in path {
            current = match accessor {
                Accessor::Field(field) => match current {
                    RecordedValue::Dict(ref dict) => dict.get(field)?.clone(),
                    _ => return None,
                },
                Accessor::Index(index) => match current {
                    RecordedValue::List(ref list) => {
                        let idx = if *index < 0 {
                            (list.len() as i64 + index) as usize
                        } else {
                            *index as usize
                        };
                        list.get(idx)?.clone()
                    }
                    _ => return None,
                },
            };
        }

        Some(current)
    }

    pub fn collect_op_dependencies(&self, value_ref: &ValueRef) -> Vec<OpId> {
        match value_ref {
            ValueRef::OpOutput { op, .. } => vec![*op],
            _ => vec![],
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::BTreeMap;

    fn setup_cache() -> OpCache {
        let cache = OpCache::new();

        cache.insert(
            OpId(0),
            RecordedValue::String("hello".to_string()),
            0,
        );

        let mut dict = BTreeMap::new();
        dict.insert("status".to_string(), RecordedValue::Int(200));
        dict.insert("body".to_string(), RecordedValue::String("response".to_string()));
        cache.insert(OpId(1), RecordedValue::Dict(dict), 0);

        cache.insert(
            OpId(2),
            RecordedValue::List(vec![
                RecordedValue::Int(1),
                RecordedValue::Int(2),
                RecordedValue::Int(3),
            ]),
            0,
        );

        cache.sync();
        cache
    }

    #[test]
    fn test_resolve_literal() {
        let cache = OpCache::new();
        let resolver = ValueResolver::new(&cache);

        let value_ref = ValueRef::literal_string("test");
        let result = resolver.resolve(&value_ref);

        assert_eq!(result, Some(RecordedValue::String("test".to_string())));
    }

    #[test]
    fn test_resolve_op_output() {
        let cache = setup_cache();
        let resolver = ValueResolver::new(&cache);

        let value_ref = ValueRef::OpOutput {
            op: OpId(0),
            path: vec![],
        };
        let result = resolver.resolve(&value_ref);

        assert_eq!(result, Some(RecordedValue::String("hello".to_string())));
    }

    #[test]
    fn test_resolve_field_access() {
        let cache = setup_cache();
        let resolver = ValueResolver::new(&cache);

        let value_ref = ValueRef::OpOutput {
            op: OpId(1),
            path: vec![Accessor::Field("status".to_string())],
        };
        let result = resolver.resolve(&value_ref);

        assert_eq!(result, Some(RecordedValue::Int(200)));
    }

    #[test]
    fn test_resolve_index_access() {
        let cache = setup_cache();
        let resolver = ValueResolver::new(&cache);

        let value_ref = ValueRef::OpOutput {
            op: OpId(2),
            path: vec![Accessor::Index(1)],
        };
        let result = resolver.resolve(&value_ref);

        assert_eq!(result, Some(RecordedValue::Int(2)));
    }

    #[test]
    fn test_resolve_to_string() {
        let cache = setup_cache();
        let resolver = ValueResolver::new(&cache);

        let value_ref = ValueRef::OpOutput {
            op: OpId(0),
            path: vec![],
        };
        let result = resolver.resolve_to_string(&value_ref);

        assert_eq!(result, Some("hello".to_string()));
    }
}
