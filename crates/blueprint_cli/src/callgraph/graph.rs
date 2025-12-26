use std::collections::HashMap;
use std::path::{Path, PathBuf};

use super::types::{CfgEdge, CfgNode, EdgeKind, NodeKind};

#[derive(Debug, Default)]
pub struct ControlFlowGraph {
    pub nodes: Vec<CfgNode>,
    pub edges: Vec<CfgEdge>,
    node_counter: usize,
}

impl ControlFlowGraph {
    pub fn new() -> Self {
        Self::default()
    }

    pub(crate) fn add_node(
        &mut self,
        kind: NodeKind,
        label: String,
        file: &Path,
        function: Option<&str>,
    ) -> usize {
        let id = self.node_counter;
        self.node_counter += 1;
        self.nodes.push(CfgNode {
            id,
            kind,
            label,
            file: file.to_path_buf(),
            function: function.map(|s| s.to_string()),
        });
        id
    }

    pub(crate) fn add_edge(&mut self, from: usize, to: usize, kind: EdgeKind) {
        self.edges.push(CfgEdge { from, to, kind });
    }

    pub fn to_dot(&self) -> String {
        let mut dot = String::new();
        dot.push_str("digraph ControlFlowGraph {\n");
        dot.push_str("    rankdir=TB;\n");
        dot.push_str("    node [fontname=\"Helvetica\", fontsize=10];\n");
        dot.push_str("    edge [fontname=\"Helvetica\", fontsize=9];\n");
        dot.push_str("\n");

        // Group nodes by function
        let mut functions: HashMap<(PathBuf, Option<String>), Vec<&CfgNode>> = HashMap::new();
        for node in &self.nodes {
            functions
                .entry((node.file.clone(), node.function.clone()))
                .or_default()
                .push(node);
        }

        // Create subgraphs for each function
        for ((file, func), nodes) in &functions {
            let file_stem = file
                .file_stem()
                .map(|s| s.to_string_lossy().to_string())
                .unwrap_or_else(|| "unknown".to_string());

            let subgraph_name = match func {
                Some(f) => format!("{}::{}", file_stem, f),
                None => format!("{}::<module>", file_stem),
            };

            dot.push_str(&format!("    subgraph \"cluster_{}\" {{\n", subgraph_name));
            dot.push_str(&format!("        label=\"{}\";\n", subgraph_name));
            dot.push_str("        style=rounded;\n");
            dot.push_str("        color=gray;\n");

            for node in nodes {
                let (shape, style, color) = match node.kind {
                    NodeKind::Entry => ("ellipse", "filled", "lightgreen"),
                    NodeKind::Exit => ("ellipse", "filled", "lightcoral"),
                    NodeKind::Statement => ("box", "rounded", "white"),
                    NodeKind::Condition => ("diamond", "filled", "lightyellow"),
                    NodeKind::ForLoop => ("hexagon", "filled", "lightblue"),
                    NodeKind::Match => ("octagon", "filled", "plum"),
                    NodeKind::Yield => ("parallelogram", "filled", "orange"),
                    NodeKind::Import => ("cds", "filled", "lightyellow"),
                    NodeKind::Export => ("cds", "filled", "lightcyan"),
                };

                let escaped_label = node
                    .label
                    .replace('\\', "\\\\")
                    .replace('"', "\\\"")
                    .replace('\n', "\\n");

                dot.push_str(&format!(
                    "        n{} [label=\"{}\" shape={} style=\"{}\" fillcolor=\"{}\"];\n",
                    node.id, escaped_label, shape, style, color
                ));
            }

            dot.push_str("    }\n\n");
        }

        // Add edges
        for edge in &self.edges {
            let (style, color, label) = match edge.kind {
                EdgeKind::Sequential => ("solid", "black", ""),
                EdgeKind::TrueBranch => ("solid", "green", "T"),
                EdgeKind::FalseBranch => ("solid", "red", "F"),
                EdgeKind::LoopBack => ("dashed", "blue", "loop"),
                EdgeKind::LoopDone => ("solid", "purple", "done"),
                EdgeKind::LoopBreak => ("bold", "red", "break"),
                EdgeKind::Call => ("dotted", "orange", "call"),
                EdgeKind::Imports => ("dashed", "purple", "from"),
                EdgeKind::Exports => ("bold", "cyan", ""),
            };

            if label.is_empty() {
                dot.push_str(&format!(
                    "    n{} -> n{} [style={} color={}];\n",
                    edge.from, edge.to, style, color
                ));
            } else {
                dot.push_str(&format!(
                    "    n{} -> n{} [style={} color={} label=\"{}\"];\n",
                    edge.from, edge.to, style, color, label
                ));
            }
        }

        dot.push_str("}\n");
        dot
    }
}
