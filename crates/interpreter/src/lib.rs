pub mod cache;
pub mod eval;
pub mod executor;
pub mod resolver;

pub use cache::{CachePolicy, CachedResult, OpCache};
pub use eval::{eval_plan, eval_plan_async, recorded_value_to_repr, recorded_value_to_string};
pub use executor::{BlueprintInterpreter, ExecutionError, ExecutionResult};
pub use resolver::ValueResolver;

// Re-export generator types for advanced usage
pub use blueprint_generator::{
    BlueprintGenerator, PlanGenerator, PlanGeneratorError,
    PlanOptimizer, PlanValidator, SchemaCache, SchemaGenerator,
    ValidationError, ValidationResult, ValidationWarning,
};
