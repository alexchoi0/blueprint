use anyhow::Result;
use clap::{Parser, Subcommand, ValueEnum};
use std::path::PathBuf;
use uuid::Uuid;

use blueprint_common::{CompiledPlan, CompiledSchema, OptLevel};
use blueprint_interpreter::BlueprintInterpreter;
use blueprint_storage::StateManager;

#[derive(Parser)]
#[command(name = "blueprint")]
#[command(about = "A Starlark execution engine with controlled system access")]
#[command(version)]
struct Cli {
    #[command(subcommand)]
    command: Commands,

    #[arg(long, global = true, default_value = "blueprint.db")]
    database: PathBuf,
}

#[derive(Clone, Copy, ValueEnum, Default)]
enum CliOptLevel {
    #[value(name = "0")]
    None,
    #[default]
    #[value(name = "1")]
    Basic,
    #[value(name = "2")]
    Aggressive,
}

impl From<CliOptLevel> for OptLevel {
    fn from(level: CliOptLevel) -> Self {
        match level {
            CliOptLevel::None => OptLevel::None,
            CliOptLevel::Basic => OptLevel::Basic,
            CliOptLevel::Aggressive => OptLevel::Aggressive,
        }
    }
}

#[derive(Subcommand)]
enum Commands {
    Run {
        script: PathBuf,

        #[arg(long)]
        dry_run: bool,
    },

    Schema {
        script: PathBuf,

        #[arg(short, long)]
        output: Option<PathBuf>,

        #[arg(long)]
        json: bool,

        #[arg(long)]
        text: bool,

        #[arg(long)]
        check: bool,
    },

    Compile {
        script: PathBuf,

        #[arg(short, long)]
        output: Option<PathBuf>,

        #[arg(short = 'O', long, value_enum, default_value = "1")]
        optimization: CliOptLevel,

        #[arg(long)]
        strip: bool,
    },

    Exec {
        plan: PathBuf,

        #[arg(long)]
        dry_run: bool,
    },

    Check {
        script: PathBuf,
    },

    Inspect {
        plan: PathBuf,

        #[arg(long)]
        json: bool,

        #[arg(long)]
        text: bool,

        #[arg(long)]
        disasm: bool,
    },

    SchemaInspect {
        schema_file: PathBuf,

        #[arg(long)]
        json: bool,

        #[arg(long)]
        text: bool,

        #[arg(long)]
        deps: bool,
    },

    #[command(subcommand)]
    State(StateCommands),

    Plan {
        script: PathBuf,

        #[arg(short, long)]
        output: Option<PathBuf>,

        #[arg(long)]
        json: bool,

        #[arg(long)]
        text: bool,

        #[arg(long)]
        dot: bool,

        #[arg(long)]
        execute: bool,
    },
}

#[derive(Subcommand)]
enum StateCommands {
    Show {
        #[arg(long)]
        plan_id: Option<Uuid>,
    },

    List,

    Op {
        op_id: i64,
    },

    ClearCache {
        #[arg(long)]
        plan_id: Option<Uuid>,

        #[arg(long)]
        op: Option<i64>,
    },

    Delete {
        #[arg(long)]
        plan_id: Uuid,
    },

    Export {
        #[arg(short, long)]
        output: Option<PathBuf>,
    },

    Import {
        #[arg(short, long)]
        input: PathBuf,
    },
}

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Commands::Run { script, dry_run } => {
            let mut interpreter = BlueprintInterpreter::new().with_dry_run(dry_run);

            if dry_run {
                let plan = interpreter.compile(&script)
                    .map_err(|e| anyhow::anyhow!("Compilation error: {:?}", e))?;
                println!("[DRY RUN] Would execute: {}", script.display());
                println!("{}", plan.display());
                return Ok(());
            }

            interpreter.run_script(&script).await
                .map_err(|e| anyhow::anyhow!("Execution error: {:?}", e))?;
        }

        Commands::Schema { script, output, json, text, check } => {
            let interpreter = BlueprintInterpreter::new();

            if check {
                interpreter.check(&script)
                    .map_err(|e| anyhow::anyhow!("Check failed: {:?}", e))?;
                println!("✓ Schema OK: {}", script.display());
                return Ok(());
            }

            let compiled = interpreter.generate_compiled_schema(&script, true)
                .map_err(|e| anyhow::anyhow!("Schema generation failed: {:?}", e))?;

            if let Some(path) = output {
                compiled.save(&path)
                    .map_err(|e| anyhow::anyhow!("Failed to save schema: {}", e))?;
                println!("Schema generated: {} -> {}", script.display(), path.display());
            } else if json {
                let output = serde_json::json!({
                    "schema_version": compiled.schema_version(),
                    "source_hash": compiled.source_hash(),
                    "compiled_at": compiled.compiled_at(),
                    "schema": compiled.schema().export_json(),
                });
                println!("{}", serde_json::to_string_pretty(&output)?);
            } else if text {
                println!("{}", compiled.to_text());
            } else {
                println!("{}", compiled.schema().display());
            }
        }

        Commands::Compile { script, output, optimization, strip } => {
            let interpreter = BlueprintInterpreter::new();
            let opt_level: OptLevel = optimization.into();
            let compiled = interpreter.generate_compiled_plan(&script, opt_level, !strip)
                .map_err(|e| anyhow::anyhow!("Compilation failed: {:?}", e))?;

            let output_path = output.unwrap_or_else(|| {
                script.with_extension("bp")
            });

            compiled.save(&output_path)
                .map_err(|e| anyhow::anyhow!("Failed to save compiled plan: {}", e))?;

            let opt_name = match opt_level {
                OptLevel::None => "none",
                OptLevel::Basic => "basic",
                OptLevel::Aggressive => "aggressive",
            };
            println!("Compiled {} -> {} (optimization: {})",
                script.display(),
                output_path.display(),
                opt_name
            );
            println!("  Schema version: {}", compiled.schema_version());
            println!("  Operations: {}", compiled.plan().len());
        }

        Commands::Exec { plan, dry_run } => {
            let compiled = CompiledPlan::load(&plan)
                .map_err(|e| anyhow::anyhow!("Failed to load compiled plan: {}", e))?;

            if dry_run {
                println!("[DRY RUN] Would execute: {}", plan.display());
                println!("{}", compiled.plan().display());
                return Ok(());
            }

            let mut interpreter = BlueprintInterpreter::new();
            interpreter.execute(compiled.plan()).await
                .map_err(|e| anyhow::anyhow!("Execution error: {:?}", e))?;
        }

        Commands::Inspect { plan, json, text, disasm } => {
            let compiled = CompiledPlan::load(&plan)
                .map_err(|e| anyhow::anyhow!("Failed to load compiled plan: {}", e))?;

            if json {
                let output = serde_json::json!({
                    "schema_version": compiled.schema_version(),
                    "source_hash": compiled.source_hash(),
                    "compiled_at": compiled.compiled_at(),
                    "optimization_level": compiled.optimization_level() as u8,
                    "plan": compiled.plan().export_json(),
                    "metadata": compiled.metadata().map(|m| serde_json::json!({
                        "source_file": m.source_file,
                        "has_source": m.source_content.is_some(),
                    })),
                });
                println!("{}", serde_json::to_string_pretty(&output)?);
            } else if text {
                println!("{}", compiled.to_text());
            } else if disasm {
                println!("=== Compiled Plan: {} ===", plan.display());
                println!("Schema version: {}", compiled.schema_version());
                println!("Source hash: {}", compiled.source_hash());
                println!("Compiled at: {}", compiled.compiled_at());
                println!("Optimization: {:?}", compiled.optimization_level());
                if let Some(meta) = compiled.metadata() {
                    if let Some(file) = &meta.source_file {
                        println!("Source: {}", file);
                    }
                }
                println!();
                println!("{}", compiled.plan().display());
            } else {
                println!("Schema: v{}", compiled.schema_version());
                println!("Hash: {}", compiled.source_hash());
                println!("Ops: {}", compiled.plan().len());
                if let Some(meta) = compiled.metadata() {
                    if let Some(file) = &meta.source_file {
                        println!("Source: {}", file);
                    }
                }
            }
        }

        Commands::SchemaInspect { schema_file, json, text, deps } => {
            let compiled = CompiledSchema::load(&schema_file)
                .map_err(|e| anyhow::anyhow!("Failed to load schema: {}", e))?;

            if json {
                let output = serde_json::json!({
                    "schema_version": compiled.schema_version(),
                    "source_hash": compiled.source_hash(),
                    "compiled_at": compiled.compiled_at(),
                    "schema": compiled.schema().export_json(),
                    "metadata": compiled.metadata().map(|m| serde_json::json!({
                        "source_file": m.source_file,
                        "has_source": m.source_content.is_some(),
                        "required_env": m.required_env,
                        "required_config": m.required_config,
                    })),
                });
                println!("{}", serde_json::to_string_pretty(&output)?);
            } else if text {
                println!("{}", compiled.to_text());
            } else if deps {
                if let Some(meta) = compiled.metadata() {
                    if !meta.required_env.is_empty() {
                        println!("Required environment variables:");
                        for env in &meta.required_env {
                            println!("  ${}", env);
                        }
                    }
                    if !meta.required_config.is_empty() {
                        println!("Required config keys:");
                        for key in &meta.required_config {
                            println!("  @{}", key);
                        }
                    }
                    if meta.required_env.is_empty() && meta.required_config.is_empty() {
                        println!("No external dependencies.");
                    }
                } else {
                    println!("No metadata available.");
                }
            } else {
                println!("Schema: v{}", compiled.schema_version());
                println!("Hash: {}", compiled.source_hash());
                println!("Ops: {}", compiled.schema().len());
                if let Some(meta) = compiled.metadata() {
                    if let Some(file) = &meta.source_file {
                        println!("Source: {}", file);
                    }
                }
            }
        }

        Commands::Check { script } => {
            let interpreter = BlueprintInterpreter::new();
            interpreter.check(&script)
                .map_err(|e| anyhow::anyhow!("Check failed: {:?}", e))?;
            println!("✓ Syntax OK: {}", script.display());
        }

        Commands::State(state_cmd) => {
            let db_path = cli.database.to_string_lossy();
            let state_manager = StateManager::new_sqlite(&db_path).await?;
            state_manager.initialize().await?;

            match state_cmd {
                StateCommands::Show { plan_id } => {
                    if let Some(id) = plan_id {
                        if let Some(summary) = state_manager.get_plan_summary(id).await? {
                            println!("Plan: {}", summary.id);
                            println!("  Name: {}", summary.name.unwrap_or_else(|| "-".to_string()));
                            println!("  Script: {}", summary.script_path);
                            println!("  Status: {:?}", summary.status);
                            println!("  Operations: {} total", summary.total_ops);
                            println!("    Pending: {}", summary.pending_ops);
                            println!("    Completed: {}", summary.completed_ops);
                            println!("    Failed: {}", summary.failed_ops);
                            println!("  Created: {}", summary.created_at);
                            println!("  Updated: {}", summary.updated_at);
                        } else {
                            println!("Plan not found: {}", id);
                        }
                    } else {
                        let plans = state_manager.list_plans().await?;
                        if plans.is_empty() {
                            println!("No plans found.");
                        } else {
                            println!("Recent plans:");
                            for plan in plans.iter().take(10) {
                                println!("  {} - {} ({:?})",
                                    plan.id,
                                    plan.name.as_deref().unwrap_or(&plan.script_path),
                                    plan.status
                                );
                            }
                        }
                    }
                }

                StateCommands::List => {
                    let plans = state_manager.list_plans().await?;
                    if plans.is_empty() {
                        println!("No plans found.");
                    } else {
                        println!("{:<36}  {:<20}  {:<12}  {}", "ID", "Name/Script", "Status", "Created");
                        println!("{}", "-".repeat(90));
                        for plan in plans {
                            println!("{:<36}  {:<20}  {:<12}  {}",
                                plan.id,
                                plan.name.as_deref().unwrap_or(&plan.script_path).chars().take(20).collect::<String>(),
                                format!("{:?}", plan.status),
                                plan.created_at.format("%Y-%m-%d %H:%M")
                            );
                        }
                    }
                }

                StateCommands::Op { op_id } => {
                    println!("Op details for {}", op_id);
                    println!("(Op lookup not yet implemented)");
                }

                StateCommands::ClearCache { plan_id, op } => {
                    if let Some(id) = plan_id {
                        let cleared = state_manager.clear_cache(id).await?;
                        println!("Cleared {} cached results for plan {}", cleared, id);
                    } else if let Some(_op_id) = op {
                        println!("(Per-op cache clearing not yet implemented)");
                    } else {
                        println!("Please specify --plan-id or --op");
                    }
                }

                StateCommands::Delete { plan_id } => {
                    state_manager.delete_plan(plan_id).await?;
                    println!("Deleted plan: {}", plan_id);
                }

                StateCommands::Export { output } => {
                    let plans = state_manager.list_plans().await?;
                    let json = serde_json::to_string_pretty(&plans)?;

                    if let Some(path) = output {
                        std::fs::write(&path, &json)?;
                        println!("Exported {} plans to {}", plans.len(), path.display());
                    } else {
                        println!("{}", json);
                    }
                }

                StateCommands::Import { input } => {
                    println!("Import from {} (not yet implemented)", input.display());
                }
            }
        }

        Commands::Plan { script, output, json, text, dot, execute } => {
            let mut interpreter = BlueprintInterpreter::new();
            let plan = interpreter.compile(&script)
                .map_err(|e| anyhow::anyhow!("Compilation failed: {:?}", e))?;

            if execute {
                println!("{}", plan.display());
                interpreter.execute(&plan).await
                    .map_err(|e| anyhow::anyhow!("Execution error: {:?}", e))?;
            } else {
                let content = if json {
                    serde_json::to_string_pretty(&plan.export_json())?
                } else if text {
                    plan.to_text()
                } else if dot {
                    plan.export_dot()
                } else {
                    plan.display()
                };

                if let Some(path) = output {
                    std::fs::write(&path, &content)?;
                    println!("Plan written to: {}", path.display());
                } else {
                    println!("{}", content);
                }
            }
        }
    }

    Ok(())
}
