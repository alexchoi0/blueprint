// Crate re-exports
pub use blueprint_approval as approval;
pub use blueprint_interpreter as interpreter;
pub use blueprint_storage as storage;

// Main entry point - BlueprintInterpreter is the primary API
pub use blueprint_interpreter::{BlueprintInterpreter, ExecutionError, ExecutionResult, OpCache};

// Generator types re-exported through interpreter for advanced users
pub use blueprint_interpreter::{
    BlueprintGenerator, PlanGenerator, PlanValidator, SchemaGenerator,
    PlanOptimizer, SchemaCache,
};

// Common types
pub use blueprint_common::{BlueprintError, BlueprintResult};
pub use blueprint_common::{Op, OpId, OpKind, Plan, Schema, SchemaOp, SchemaOpId, SchemaValue};
pub use blueprint_common::{ExecutionContext, OptLevel, CompiledPlan, CompiledSchema};

// Storage types
pub use blueprint_storage::{StateManager, PlanStatus, OpStatus};
