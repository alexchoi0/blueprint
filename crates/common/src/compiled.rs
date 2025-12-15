use serde::{Deserialize, Serialize};
use std::fs::File;
use std::io::{BufReader, BufWriter, Read, Write};
use std::path::Path;
use std::time::{SystemTime, UNIX_EPOCH};

use crate::OptLevel;
use crate::plan::Plan;
use crate::PLAN_SCHEMA_VERSION;

const MAGIC: [u8; 4] = [b'B', b'P', 0x00, 0x01];

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompiledPlan {
    schema_version: u32,
    source_hash: String,
    compiled_at: u64,
    optimization_level: u8,
    plan: Plan,
    metadata: Option<PlanMetadata>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlanMetadata {
    pub source_file: Option<String>,
    pub source_content: Option<String>,
}

#[derive(Debug)]
pub enum CompiledPlanError {
    Io(std::io::Error),
    InvalidMagic,
    SchemaMismatch { expected: u32, found: u32 },
    SerializationError(String),
}

impl std::fmt::Display for CompiledPlanError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            CompiledPlanError::Io(e) => write!(f, "IO error: {}", e),
            CompiledPlanError::InvalidMagic => write!(f, "Invalid file format: not a .bp file"),
            CompiledPlanError::SchemaMismatch { expected, found } => {
                write!(
                    f,
                    "Schema version mismatch: expected {}, found {}. Please recompile the .star file.",
                    expected, found
                )
            }
            CompiledPlanError::SerializationError(msg) => write!(f, "Serialization error: {}", msg),
        }
    }
}

impl std::error::Error for CompiledPlanError {}

impl From<std::io::Error> for CompiledPlanError {
    fn from(e: std::io::Error) -> Self {
        CompiledPlanError::Io(e)
    }
}

impl CompiledPlan {
    pub fn new(
        plan: Plan,
        source_hash: String,
        opt_level: OptLevel,
        metadata: Option<PlanMetadata>,
    ) -> Self {
        let compiled_at = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();

        CompiledPlan {
            schema_version: PLAN_SCHEMA_VERSION,
            source_hash,
            compiled_at,
            optimization_level: opt_level as u8,
            plan,
            metadata,
        }
    }

    pub fn plan(&self) -> &Plan {
        &self.plan
    }

    pub fn into_plan(self) -> Plan {
        self.plan
    }

    pub fn source_hash(&self) -> &str {
        &self.source_hash
    }

    pub fn compiled_at(&self) -> u64 {
        self.compiled_at
    }

    pub fn optimization_level(&self) -> OptLevel {
        match self.optimization_level {
            0 => OptLevel::None,
            1 => OptLevel::Basic,
            _ => OptLevel::Aggressive,
        }
    }

    pub fn metadata(&self) -> Option<&PlanMetadata> {
        self.metadata.as_ref()
    }

    pub fn schema_version(&self) -> u32 {
        self.schema_version
    }

    pub fn save<P: AsRef<Path>>(&self, path: P) -> Result<(), CompiledPlanError> {
        let file = File::create(path)?;
        let mut writer = BufWriter::new(file);

        writer.write_all(&MAGIC)?;

        let encoded = bincode::serialize(self)
            .map_err(|e| CompiledPlanError::SerializationError(e.to_string()))?;
        writer.write_all(&encoded)?;

        Ok(())
    }

    pub fn load<P: AsRef<Path>>(path: P) -> Result<Self, CompiledPlanError> {
        let file = File::open(path)?;
        let mut reader = BufReader::new(file);

        let mut magic = [0u8; 4];
        reader.read_exact(&mut magic)?;
        if magic != MAGIC {
            return Err(CompiledPlanError::InvalidMagic);
        }

        let mut data = Vec::new();
        reader.read_to_end(&mut data)?;

        let compiled: CompiledPlan = bincode::deserialize(&data)
            .map_err(|e| CompiledPlanError::SerializationError(e.to_string()))?;

        if compiled.schema_version != PLAN_SCHEMA_VERSION {
            return Err(CompiledPlanError::SchemaMismatch {
                expected: PLAN_SCHEMA_VERSION,
                found: compiled.schema_version,
            });
        }

        Ok(compiled)
    }

    pub fn to_bytes(&self) -> Result<Vec<u8>, CompiledPlanError> {
        let mut bytes = Vec::with_capacity(MAGIC.len() + 1024);
        bytes.extend_from_slice(&MAGIC);

        let encoded = bincode::serialize(self)
            .map_err(|e| CompiledPlanError::SerializationError(e.to_string()))?;
        bytes.extend(encoded);

        Ok(bytes)
    }

    pub fn from_bytes(bytes: &[u8]) -> Result<Self, CompiledPlanError> {
        if bytes.len() < MAGIC.len() {
            return Err(CompiledPlanError::InvalidMagic);
        }

        if bytes[..MAGIC.len()] != MAGIC {
            return Err(CompiledPlanError::InvalidMagic);
        }

        let compiled: CompiledPlan = bincode::deserialize(&bytes[MAGIC.len()..])
            .map_err(|e| CompiledPlanError::SerializationError(e.to_string()))?;

        if compiled.schema_version != PLAN_SCHEMA_VERSION {
            return Err(CompiledPlanError::SchemaMismatch {
                expected: PLAN_SCHEMA_VERSION,
                found: compiled.schema_version,
            });
        }

        Ok(compiled)
    }

    pub fn to_text(&self) -> String {
        let mut out = String::new();
        out.push_str("; Blueprint Plan\n");
        if let Some(meta) = &self.metadata {
            if let Some(src) = &meta.source_file {
                out.push_str(&format!("; Source: {}\n", src));
            }
        }
        out.push_str(&format!("; Hash: {}\n", self.source_hash));
        out.push_str(&format!("; Version: {}\n", self.schema_version));
        out.push_str(&format!("; OptLevel: {:?}\n", self.optimization_level()));
        out.push_str(&format!("; Operations: {}\n\n", self.plan.len()));

        out.push_str(&self.plan.to_text());

        out
    }
}

pub fn compute_source_hash(source: &str) -> String {
    use sha2::{Digest, Sha256};
    let mut hasher = Sha256::new();
    hasher.update(source.as_bytes());
    format!("{:x}", hasher.finalize())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::op::{OpKind, ValueRef};
    use tempfile::tempdir;

    #[test]
    fn test_compiled_plan_save_load() {
        let mut plan = Plan::new();
        plan.add_op(
            OpKind::Print {
                message: ValueRef::literal_string("hello"),
            },
            None,
        );

        let source = "print('hello')";
        let hash = compute_source_hash(source);

        let compiled = CompiledPlan::new(
            plan.clone(),
            hash.clone(),
            OptLevel::Basic,
            Some(PlanMetadata {
                source_file: Some("test.star".to_string()),
                source_content: Some(source.to_string()),
            }),
        );

        let dir = tempdir().unwrap();
        let path = dir.path().join("test.bp");

        compiled.save(&path).unwrap();
        let loaded = CompiledPlan::load(&path).unwrap();

        assert_eq!(loaded.source_hash(), hash);
        assert_eq!(loaded.plan().len(), 1);
        assert!(loaded.metadata().is_some());
        assert_eq!(loaded.schema_version(), PLAN_SCHEMA_VERSION);
    }

    #[test]
    fn test_compiled_plan_bytes() {
        let mut plan = Plan::new();
        plan.add_op(
            OpKind::Print {
                message: ValueRef::literal_string("test"),
            },
            None,
        );

        let compiled = CompiledPlan::new(plan, "abc123".to_string(), OptLevel::None, None);

        let bytes = compiled.to_bytes().unwrap();
        assert!(bytes.starts_with(&MAGIC));

        let loaded = CompiledPlan::from_bytes(&bytes).unwrap();
        assert_eq!(loaded.source_hash(), "abc123");
    }

    #[test]
    fn test_invalid_magic() {
        let bad_bytes = [0u8, 1, 2, 3, 4, 5];
        let result = CompiledPlan::from_bytes(&bad_bytes);
        assert!(matches!(result, Err(CompiledPlanError::InvalidMagic)));
    }

    #[test]
    fn test_source_hash() {
        let hash1 = compute_source_hash("hello");
        let hash2 = compute_source_hash("hello");
        let hash3 = compute_source_hash("world");

        assert_eq!(hash1, hash2);
        assert_ne!(hash1, hash3);
        assert_eq!(hash1.len(), 64);
    }
}
