pub mod compiled;
pub mod context;
pub mod error;
pub mod op;
pub mod plan;
pub mod schema;

pub use compiled::{CompiledPlan, CompiledPlanError, PlanMetadata, compute_source_hash};
pub use context::{ExecutionContext, ProjectConfig, PathMapping, ConfigError};
pub use error::{BlueprintError, BlueprintResult};
pub use op::{Op, OpId, OpKind, RecordedValue, SourceSpan, ValueRef, SubPlan, Accessor};
pub use plan::{Plan, CycleError};
pub use schema::{
    Schema, SchemaEntry, SchemaOp, SchemaOpId, SchemaValue,
    SchemaSubPlan, SchemaSubPlanEntry,
    CompiledSchema, CompiledSchemaError, SchemaMetadata, SCHEMA_VERSION,
};

/// Schema version for Plan serialization.
pub const PLAN_SCHEMA_VERSION: u32 = 5;

/// Optimization level for plan compilation.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, serde::Serialize, serde::Deserialize)]
#[repr(u8)]
pub enum OptLevel {
    None = 0,
    #[default]
    Basic = 1,
    Aggressive = 2,
}
