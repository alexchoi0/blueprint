mod builder;
mod graph;
mod types;

pub use graph::ControlFlowGraph;

use builder::CfgBuilder;
use std::path::PathBuf;

pub fn analyze_files(files: &[PathBuf]) -> ControlFlowGraph {
    let mut builder = CfgBuilder::new();

    for file in files {
        let content = match std::fs::read_to_string(file) {
            Ok(c) => c,
            Err(_) => continue,
        };

        let filename = file.to_string_lossy().to_string();
        let module = match blueprint_engine_parser::parse(&filename, &content) {
            Ok(m) => m,
            Err(_) => continue,
        };

        builder.analyze_file(file, &module);
    }

    builder.link_imports();
    builder.build()
}
