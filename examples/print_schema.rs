use blueprint_interpreter::SchemaGenerator;
use blueprint_common::SchemaOp;

fn main() {
    let result = SchemaGenerator::generate(
        r#"
load("@bp/io", "read_file", "write_file")

def process_files():
    files = [read_file("a.txt"), read_file("b.txt")]
    for content in files:
        write_file("out.txt", content)
process_files()
"#,
        "test.star",
    );
    
    match result {
        Ok(schema) => {
            println!("╔════════════════════════════════════════════════════════════════╗");
            println!("║                    COMPILED SCHEMA ({} ops)                     ║", schema.len());
            println!("╚════════════════════════════════════════════════════════════════╝\n");
            
            for entry in schema.entries.iter() {
                println!("┌─ Op [{}] ─────────────────────────────────────", entry.id.0);
                println!("│  Type: {}", entry.op.name());
                println!("│  Inputs: {:?}", entry.inputs);
                
                match &entry.op {
                    SchemaOp::ForEach { items, item_name, body, parallel } => {
                        println!("│  Items: {}", items);
                        println!("│  Item name: {}", item_name);
                        println!("│  Parallel: {}", parallel);
                        println!("│  ┌─ SubPlan ────────────────────────────");
                        println!("│  │  Params: {:?}", body.params);
                        println!("│  │  Output: {}", body.output);
                        for sub_entry in &body.entries {
                            println!("│  │  [{}] {} (guard: {:?})", 
                                sub_entry.local_id, 
                                sub_entry.op, 
                                sub_entry.guard);
                        }
                        println!("│  └────────────────────────────────────────");
                    }
                    SchemaOp::IfBlock { condition, then_body, else_body } => {
                        println!("│  Condition: {}", condition);
                        println!("│  Then body: {} ops", then_body.entries.len());
                        if let Some(eb) = else_body {
                            println!("│  Else body: {} ops", eb.entries.len());
                        }
                    }
                    _ => {
                        println!("│  Details: {}", entry.op);
                    }
                }
                println!("└──────────────────────────────────────────────────\n");
            }
            
            println!("\n=== Execution Order ===");
            println!("1. Op[0] and Op[1] can execute in parallel (no dependencies)");
            println!("2. Op[2] (ForEach) waits for items to be ready");
            println!("3. For each item, execute SubPlan body sequentially");
        }
        Err(e) => {
            eprintln!("Compilation error: {}", e);
        }
    }
}
