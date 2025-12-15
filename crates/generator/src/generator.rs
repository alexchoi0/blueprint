use crate::starlark::SchemaGenerator;
use blueprint_common::{
    CompiledPlan, CompiledSchema, OptLevel, Plan, PlanMetadata, Schema,
    compute_source_hash, PLAN_SCHEMA_VERSION,
};
use blueprint_storage::StateManager;
use crate::optimizer::PlanOptimizer;
use anyhow::Result;
use lru::LruCache;
use starlark_syntax::syntax::{module::AstModule, Dialect};
use std::num::NonZeroUsize;
use std::path::Path;
use std::sync::{Arc, Mutex};

pub fn blueprint_dialect() -> Dialect {
    Dialect::Extended
}

const DEFAULT_SCHEMA_CACHE_CAPACITY: usize = 100;

pub struct SchemaCache {
    cache: LruCache<String, Schema>,
}

impl SchemaCache {
    pub fn new() -> Self {
        Self::with_capacity(DEFAULT_SCHEMA_CACHE_CAPACITY)
    }

    pub fn with_capacity(capacity: usize) -> Self {
        Self {
            cache: LruCache::new(NonZeroUsize::new(capacity).unwrap()),
        }
    }

    pub fn get(&mut self, script_hash: &str) -> Option<Schema> {
        self.cache.get(script_hash).cloned()
    }

    pub fn insert(&mut self, script_hash: String, schema: Schema) {
        self.cache.put(script_hash, schema);
    }

    pub fn len(&self) -> usize {
        self.cache.len()
    }

    pub fn clear(&mut self) {
        self.cache.clear();
    }

    pub fn compute_hash(content: &str) -> String {
        let script_hash = StateManager::compute_script_hash(content);
        format!("v{}:{}", PLAN_SCHEMA_VERSION, script_hash)
    }
}

impl Default for SchemaCache {
    fn default() -> Self {
        Self::new()
    }
}

pub struct BlueprintGenerator {
    schema_cache: Arc<Mutex<SchemaCache>>,
}

impl BlueprintGenerator {
    pub fn new() -> Self {
        Self {
            schema_cache: Arc::new(Mutex::new(SchemaCache::new())),
        }
    }

    pub fn schema_cache(&self) -> Arc<Mutex<SchemaCache>> {
        Arc::clone(&self.schema_cache)
    }

    pub fn check(&self, path: &Path) -> Result<()> {
        let content = std::fs::read_to_string(path)?;
        let filename = path
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("script.star");

        AstModule::parse(filename, content, &blueprint_dialect())
            .map_err(|e| anyhow::anyhow!("Parse error: {}", e))?;
        Ok(())
    }

    pub fn generate_from_source(&self, source: &str) -> Result<Schema> {
        SchemaGenerator::generate(source, "eval.star")
            .map_err(|e| anyhow::anyhow!("Schema generation error: {}", e))
    }

    pub fn generate_schema(&self, path: &Path) -> Result<Schema> {
        let content = std::fs::read_to_string(path)?;
        let filename = path
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("script.star");

        let hash = SchemaCache::compute_hash(&content);

        if let Some(cached) = self.schema_cache.lock().unwrap().get(&hash) {
            return Ok(cached);
        }

        let schema = SchemaGenerator::generate(&content, filename)
            .map_err(|e| anyhow::anyhow!("Schema generation error: {}", e))?;

        self.schema_cache.lock().unwrap().insert(hash, schema.clone());
        Ok(schema)
    }

    pub fn generate_compiled_schema(&self, path: &Path, include_source: bool) -> Result<CompiledSchema> {
        let content = std::fs::read_to_string(path)?;
        let schema = self.generate_schema(path)?;
        let source_hash = compute_source_hash(&content);

        let metadata = if include_source {
            Some(blueprint_common::SchemaMetadata {
                source_file: Some(path.to_string_lossy().to_string()),
                source_content: Some(content),
                required_env: Vec::new(),
                required_config: Vec::new(),
            })
        } else {
            Some(blueprint_common::SchemaMetadata {
                source_file: Some(path.to_string_lossy().to_string()),
                source_content: None,
                required_env: Vec::new(),
                required_config: Vec::new(),
            })
        };

        Ok(CompiledSchema::new(schema, source_hash, metadata))
    }

    pub fn generate_compiled_plan(&self, path: &Path, plan: Plan, opt_level: OptLevel, include_source: bool) -> Result<CompiledPlan> {
        let content = std::fs::read_to_string(path)?;

        let optimizer = PlanOptimizer::new(opt_level);
        let optimized_plan = optimizer.optimize(plan);

        let source_hash = compute_source_hash(&content);
        let metadata = if include_source {
            Some(PlanMetadata {
                source_file: Some(path.to_string_lossy().to_string()),
                source_content: Some(content),
            })
        } else {
            Some(PlanMetadata {
                source_file: Some(path.to_string_lossy().to_string()),
                source_content: None,
            })
        };

        Ok(CompiledPlan::new(optimized_plan, source_hash, opt_level, metadata))
    }
}

impl Default for BlueprintGenerator {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::resolver::PlanGenerator;
    use blueprint_common::ExecutionContext;

    #[test]
    fn test_generate_from_source() {
        let generator = BlueprintGenerator::new();
        let schema = generator.generate_from_source(r#"
load("@bp/io", "write_file")
write_file("/tmp/test.txt", "hello")
"#).expect("schema generation failed");
        assert_eq!(schema.entries.len(), 1);
    }

    #[test]
    fn test_generate_schema_and_plan() {
        let generator = BlueprintGenerator::new();
        let schema = generator.generate_from_source(r#"
load("@bp/io", "write_file")
write_file("/tmp/test.txt", "hello")
"#).expect("schema generation failed");

        let ctx = ExecutionContext::from_current_env();
        let plan_gen = PlanGenerator::new(&ctx);
        let plan = plan_gen.generate(&schema).expect("plan generation failed");
        assert_eq!(plan.ops().count(), 1);
    }
}
