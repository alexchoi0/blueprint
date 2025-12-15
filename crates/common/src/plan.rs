use indexmap::{IndexMap, IndexSet};

use super::op::{Op, OpId, OpKind};

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct Plan {
    ops: Vec<Op>,
    next_id: u64,
}

impl Plan {
    pub fn new() -> Self {
        Self {
            ops: Vec::new(),
            next_id: 0,
        }
    }

    pub fn add_op(&mut self, kind: OpKind, source_location: Option<super::op::SourceSpan>) -> OpId {
        let id = OpId(self.next_id);
        self.next_id += 1;

        // For After { dependency, value }, inject dependency into value's inputs
        // This ensures value op won't start until dependency completes
        if let OpKind::After { dependency, value } = &kind {
            if let Some(value_op) = self.ops.iter_mut().find(|op| op.id == *value) {
                if !value_op.inputs.contains(dependency) {
                    value_op.inputs.push(*dependency);
                }
            }
        }

        let inputs = self.compute_inputs(&kind);

        let op = Op {
            id,
            kind,
            inputs,
            source_location,
            guard: None,
        };

        self.ops.push(op);
        id
    }

    fn compute_inputs(&self, kind: &OpKind) -> Vec<OpId> {
        let mut inputs = IndexSet::new();

        for value_ref in kind.collect_value_refs() {
            if let Some(op_id) = value_ref.referenced_op() {
                inputs.insert(op_id);
            }
        }

        for op_id in kind.collect_op_refs() {
            inputs.insert(op_id);
        }

        inputs.into_iter().collect()
    }

    pub fn get_op(&self, id: OpId) -> Option<&Op> {
        self.ops.iter().find(|op| op.id == id)
    }

    pub fn get_op_mut(&mut self, id: OpId) -> Option<&mut Op> {
        self.ops.iter_mut().find(|op| op.id == id)
    }

    pub fn ops(&self) -> impl Iterator<Item = &Op> {
        self.ops.iter()
    }

    pub fn ops_mut(&mut self) -> impl Iterator<Item = &mut Op> {
        self.ops.iter_mut()
    }

    pub fn len(&self) -> usize {
        self.ops.len()
    }

    pub fn is_empty(&self) -> bool {
        self.ops.is_empty()
    }

    pub fn remove_ops<F>(&mut self, should_remove: F)
    where
        F: Fn(&Op) -> bool,
    {
        self.ops.retain(|op| !should_remove(op));
    }

    pub fn compute_levels(&self) -> Result<Vec<Vec<OpId>>, CycleError> {
        let mut in_degree: IndexMap<OpId, usize> = IndexMap::new();
        let mut dependents: IndexMap<OpId, Vec<OpId>> = IndexMap::new();

        for op in &self.ops {
            in_degree.entry(op.id).or_insert(0);
            for &input in &op.inputs {
                *in_degree.entry(op.id).or_insert(0) += 1;
                dependents.entry(input).or_default().push(op.id);
            }
        }

        let mut levels: Vec<Vec<OpId>> = Vec::new();
        let mut current_level: Vec<OpId> = in_degree
            .iter()
            .filter(|(_, &deg)| deg == 0)
            .map(|(&id, _)| id)
            .collect();

        let mut processed = 0;

        while !current_level.is_empty() {
            current_level.sort_by_key(|id| id.0);
            levels.push(current_level.clone());
            processed += current_level.len();

            let mut next_level = Vec::new();
            for id in &current_level {
                if let Some(deps) = dependents.get(id) {
                    for &dep_id in deps {
                        let deg = in_degree.get_mut(&dep_id).unwrap();
                        *deg -= 1;
                        if *deg == 0 {
                            next_level.push(dep_id);
                        }
                    }
                }
            }
            current_level = next_level;
        }

        if processed != self.ops.len() {
            let remaining: Vec<OpId> = in_degree
                .iter()
                .filter(|(_, &deg)| deg > 0)
                .map(|(&id, _)| id)
                .collect();
            return Err(CycleError { ops: remaining });
        }

        Ok(levels)
    }

    pub fn ops_requiring_approval(&self) -> Vec<&Op> {
        self.ops.iter().filter(|op| op.kind.requires_approval()).collect()
    }

    pub fn display(&self) -> String {
        let mut output = String::new();

        match self.compute_levels() {
            Ok(levels) => {
                output.push_str("═══════════════════════════════════════════════════════════════\n");
                output.push_str("                         EXECUTION PLAN\n");
                output.push_str("═══════════════════════════════════════════════════════════════\n\n");

                for (level_idx, level) in levels.iter().enumerate() {
                    let parallel = level.len() > 1;
                    if parallel {
                        output.push_str(&format!("Level {} (PARALLEL):\n", level_idx));
                    } else {
                        output.push_str(&format!("Level {}:\n", level_idx));
                    }

                    for op_id in level {
                        if let Some(op) = self.get_op(*op_id) {
                            let approval = if op.kind.requires_approval() { " [requires approval]" } else { "" };
                            output.push_str(&format!("  [{}] {}{}\n", op_id.0, op.kind, approval));
                        }
                    }
                    output.push('\n');
                }

                let approval_count = self.ops_requiring_approval().len();
                output.push_str("───────────────────────────────────────────────────────────────\n");
                output.push_str(&format!("  Operations: {} total, {} require approval\n", self.ops.len(), approval_count));
            }
            Err(cycle) => {
                output.push_str("ERROR: Cycle detected in plan!\n");
                output.push_str(&format!("  Involved ops: {:?}\n", cycle.ops.iter().map(|o| o.0).collect::<Vec<_>>()));
            }
        }

        output
    }

    pub fn export_dot(&self) -> String {
        let mut output = String::from("digraph Plan {\n");
        output.push_str("  rankdir=TB;\n");
        output.push_str("  node [shape=box, style=rounded];\n\n");

        for op in &self.ops {
            let color = if op.kind.requires_approval() { "lightcoral" } else { "lightblue" };
            let label = format!("[{}] {}", op.id.0, op.kind.name());
            output.push_str(&format!(
                "  op{} [label=\"{}\", fillcolor={}, style=filled];\n",
                op.id.0, label, color
            ));
        }

        output.push('\n');

        for op in &self.ops {
            for input in &op.inputs {
                output.push_str(&format!("  op{} -> op{};\n", input.0, op.id.0));
            }
        }

        output.push_str("}\n");
        output
    }

    pub fn export_json(&self) -> serde_json::Value {
        serde_json::json!({
            "ops": self.ops.iter().map(|op| {
                serde_json::json!({
                    "id": op.id.0,
                    "kind": op.kind.name(),
                    "inputs": op.inputs.iter().map(|i| i.0).collect::<Vec<_>>(),
                    "requires_approval": op.kind.requires_approval(),
                })
            }).collect::<Vec<_>>(),
            "levels": self.compute_levels().ok().map(|levels| {
                levels.iter().map(|level| {
                    level.iter().map(|id| id.0).collect::<Vec<_>>()
                }).collect::<Vec<_>>()
            }),
        })
    }

    pub fn to_text(&self) -> String {
        let mut out = String::new();
        out.push_str(".section plan\n\n");

        for op in self.ops() {
            let mut comments = Vec::new();
            if op.kind.requires_approval() {
                comments.push("[approval]".to_string());
            }
            if !op.inputs.is_empty() {
                let deps: Vec<String> = op.inputs.iter().map(|i| format!("@{}", i.0)).collect();
                comments.push(format!("after {}", deps.join(", ")));
            }

            let comment = if comments.is_empty() {
                String::new()
            } else {
                format!("  ; {}", comments.join(" "))
            };

            out.push_str(&format!("@{}: {}{}\n", op.id.0, op.kind.name(), comment));

            for (name, value) in op.kind.to_text_fields() {
                out.push_str(&format!("    {:12} = {}\n", name, value));
            }
            out.push('\n');
        }

        out.push_str(".section summary\n");
        out.push_str(&format!("    total_ops       = {}\n", self.len()));
        out.push_str(&format!("    approval_needed = {}\n", self.ops_requiring_approval().len()));

        out
    }
}

impl Default for Plan {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug, Clone)]
pub struct CycleError {
    pub ops: Vec<OpId>,
}

impl std::fmt::Display for CycleError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "Cycle detected in plan involving ops: {:?}",
            self.ops.iter().map(|o| o.0).collect::<Vec<_>>()
        )
    }
}

impl std::error::Error for CycleError {}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::op::ValueRef;

    #[test]
    fn test_empty_plan() {
        let plan = Plan::new();
        assert!(plan.is_empty());
        assert_eq!(plan.len(), 0);
    }

    #[test]
    fn test_add_op() {
        let mut plan = Plan::new();
        let op_id = plan.add_op(OpKind::Now, None);
        assert_eq!(op_id.0, 0);
        assert_eq!(plan.len(), 1);
    }

    #[test]
    fn test_dependency_tracking() {
        let mut plan = Plan::new();

        let read_id = plan.add_op(
            OpKind::ReadFile {
                path: ValueRef::literal_string("config.json"),
            },
            None,
        );

        let decode_id = plan.add_op(
            OpKind::JsonDecode {
                string: ValueRef::op_output(read_id),
            },
            None,
        );

        let decode_op = plan.get_op(decode_id).unwrap();
        assert!(decode_op.inputs.contains(&read_id));
    }

    #[test]
    fn test_compute_levels() {
        let mut plan = Plan::new();

        let op0 = plan.add_op(OpKind::Now, None);

        let op1 = plan.add_op(
            OpKind::Sleep {
                seconds: ValueRef::op_output(op0),
            },
            None,
        );

        let op2 = plan.add_op(
            OpKind::Print {
                message: ValueRef::op_output(op1),
            },
            None,
        );

        let levels = plan.compute_levels().unwrap();
        assert_eq!(levels.len(), 3);
        assert!(levels[0].contains(&op0));
        assert!(levels[1].contains(&op1));
        assert!(levels[2].contains(&op2));
    }

    #[test]
    fn test_parallel_detection() {
        let mut plan = Plan::new();

        let _read1 = plan.add_op(
            OpKind::ReadFile {
                path: ValueRef::literal_string("file1.txt"),
            },
            None,
        );

        let _read2 = plan.add_op(
            OpKind::ReadFile {
                path: ValueRef::literal_string("file2.txt"),
            },
            None,
        );

        let levels = plan.compute_levels().unwrap();
        assert_eq!(levels.len(), 1);
        assert_eq!(levels[0].len(), 2);
    }

    fn make_simple_subplan() -> crate::op::SubPlan {
        crate::op::SubPlan {
            params: vec!["x".to_string()],
            ops: vec![Op {
                id: OpId(0),
                kind: OpKind::Print {
                    message: ValueRef::Dynamic("x".to_string()),
                },
                inputs: vec![],
                source_location: None,
                guard: None,
            }],
            output: OpId(0),
        }
    }

    #[test]
    fn test_plan_with_foreach() {
        let mut plan = Plan::new();

        let read_id = plan.add_op(
            OpKind::ReadFile {
                path: ValueRef::literal_string("data.txt"),
            },
            None,
        );

        let foreach_id = plan.add_op(
            OpKind::ForEach {
                items: ValueRef::op_output(read_id),
                item_name: "line".to_string(),
                body: make_simple_subplan(),
                parallel: false,
            },
            None,
        );

        assert_eq!(plan.len(), 2);
        let foreach_op = plan.get_op(foreach_id).unwrap();
        assert!(foreach_op.inputs.contains(&read_id));
    }

    #[test]
    fn test_plan_with_ifblock() {
        let mut plan = Plan::new();

        let cond_id = plan.add_op(
            OpKind::FileExists {
                path: ValueRef::literal_string("test.txt"),
            },
            None,
        );

        let ifblock_id = plan.add_op(
            OpKind::IfBlock {
                condition: ValueRef::op_output(cond_id),
                then_body: make_simple_subplan(),
                else_body: None,
            },
            None,
        );

        assert_eq!(plan.len(), 2);
        let ifblock_op = plan.get_op(ifblock_id).unwrap();
        assert!(ifblock_op.inputs.contains(&cond_id));
    }

    #[test]
    fn test_plan_with_break_continue() {
        let mut plan = Plan::new();

        let break_id = plan.add_op(OpKind::Break, None);
        let continue_id = plan.add_op(OpKind::Continue, None);

        assert_eq!(plan.len(), 2);
        let break_op = plan.get_op(break_id).unwrap();
        let continue_op = plan.get_op(continue_id).unwrap();

        assert!(break_op.inputs.is_empty());
        assert!(continue_op.inputs.is_empty());
    }

    #[test]
    fn test_plan_foreach_levels() {
        let mut plan = Plan::new();

        let read_id = plan.add_op(
            OpKind::ReadFile {
                path: ValueRef::literal_string("data.txt"),
            },
            None,
        );

        let foreach_id = plan.add_op(
            OpKind::ForEach {
                items: ValueRef::op_output(read_id),
                item_name: "item".to_string(),
                body: make_simple_subplan(),
                parallel: false,
            },
            None,
        );

        let levels = plan.compute_levels().unwrap();
        assert_eq!(levels.len(), 2);
        assert!(levels[0].contains(&read_id));
        assert!(levels[1].contains(&foreach_id));
    }

    #[test]
    fn test_plan_ifblock_levels() {
        let mut plan = Plan::new();

        let cond_id = plan.add_op(
            OpKind::FileExists {
                path: ValueRef::literal_string("test.txt"),
            },
            None,
        );

        let ifblock_id = plan.add_op(
            OpKind::IfBlock {
                condition: ValueRef::op_output(cond_id),
                then_body: make_simple_subplan(),
                else_body: Some(make_simple_subplan()),
            },
            None,
        );

        let levels = plan.compute_levels().unwrap();
        assert_eq!(levels.len(), 2);
        assert!(levels[0].contains(&cond_id));
        assert!(levels[1].contains(&ifblock_id));
    }

    #[test]
    fn test_plan_parallel_foreach_ops() {
        let mut plan = Plan::new();

        let foreach1_id = plan.add_op(
            OpKind::ForEach {
                items: ValueRef::Literal(crate::op::RecordedValue::List(vec![
                    crate::op::RecordedValue::Int(1),
                    crate::op::RecordedValue::Int(2),
                ])),
                item_name: "a".to_string(),
                body: make_simple_subplan(),
                parallel: true,
            },
            None,
        );

        let foreach2_id = plan.add_op(
            OpKind::ForEach {
                items: ValueRef::Literal(crate::op::RecordedValue::List(vec![
                    crate::op::RecordedValue::Int(3),
                    crate::op::RecordedValue::Int(4),
                ])),
                item_name: "b".to_string(),
                body: make_simple_subplan(),
                parallel: true,
            },
            None,
        );

        let levels = plan.compute_levels().unwrap();
        assert_eq!(levels.len(), 1);
        assert!(levels[0].contains(&foreach1_id));
        assert!(levels[0].contains(&foreach2_id));
    }

    #[test]
    fn test_plan_serialization_with_control_flow() {
        let mut plan = Plan::new();

        let read_id = plan.add_op(
            OpKind::ReadFile {
                path: ValueRef::literal_string("data.txt"),
            },
            None,
        );

        plan.add_op(
            OpKind::ForEach {
                items: ValueRef::op_output(read_id),
                item_name: "line".to_string(),
                body: make_simple_subplan(),
                parallel: true,
            },
            None,
        );

        let serialized = bincode::serialize(&plan).expect("serialization failed");
        let deserialized: Plan = bincode::deserialize(&serialized).expect("deserialization failed");

        assert_eq!(deserialized.len(), 2);

        let foreach_op = deserialized.ops().nth(1).unwrap();
        if let OpKind::ForEach { item_name, parallel, .. } = &foreach_op.kind {
            assert_eq!(item_name, "line");
            assert!(parallel);
        } else {
            panic!("Expected ForEach op");
        }
    }

    #[test]
    fn test_plan_serialization_with_ifblock() {
        let mut plan = Plan::new();

        plan.add_op(
            OpKind::IfBlock {
                condition: ValueRef::Literal(crate::op::RecordedValue::Bool(true)),
                then_body: make_simple_subplan(),
                else_body: Some(make_simple_subplan()),
            },
            None,
        );

        let serialized = bincode::serialize(&plan).expect("serialization failed");
        let deserialized: Plan = bincode::deserialize(&serialized).expect("deserialization failed");

        assert_eq!(deserialized.len(), 1);

        let ifblock_op = deserialized.ops().next().unwrap();
        if let OpKind::IfBlock { else_body, .. } = &ifblock_op.kind {
            assert!(else_body.is_some());
        } else {
            panic!("Expected IfBlock op");
        }
    }
}
