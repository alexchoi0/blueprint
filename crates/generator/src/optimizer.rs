use std::collections::HashMap;
use blueprint_common::{OpId, OpKind, RecordedValue, ValueRef, Plan, OptLevel};

pub struct PlanOptimizer {
    level: OptLevel,
}

impl PlanOptimizer {
    pub fn new(level: OptLevel) -> Self {
        Self { level }
    }

    pub fn optimize(&self, mut plan: Plan) -> Plan {
        match self.level {
            OptLevel::None => plan,
            OptLevel::Basic => self.constant_fold(plan),
            OptLevel::Aggressive => {
                plan = self.constant_fold(plan);
                self.dead_code_eliminate(plan)
            }
        }
    }

    fn constant_fold(&self, mut plan: Plan) -> Plan {
        let mut folded_values: HashMap<OpId, RecordedValue> = HashMap::new();

        loop {
            let mut changed = false;

            for op in plan.ops_mut() {
                self.substitute_folded_refs(&mut op.kind, &folded_values);

                if !folded_values.contains_key(&op.id) && op.kind.can_fold() {
                    if let Some(result) = self.evaluate_pure(&op.kind) {
                        folded_values.insert(op.id, result);
                        changed = true;
                    }
                }
            }

            if !changed {
                break;
            }
        }

        self.remove_folded_ops(&mut plan, &folded_values);
        plan
    }

    fn substitute_folded_refs(&self, kind: &mut OpKind, folded: &HashMap<OpId, RecordedValue>) {
        for value_ref in kind.collect_value_refs_mut() {
            self.substitute_ref(value_ref, folded);
        }
    }

    fn substitute_ref(&self, value_ref: &mut ValueRef, folded: &HashMap<OpId, RecordedValue>) {
        if let ValueRef::OpOutput { op, path } = value_ref {
            if path.is_empty() {
                if let Some(val) = folded.get(op) {
                    *value_ref = ValueRef::Literal(val.clone());
                }
            }
        }
    }

    fn evaluate_pure(&self, kind: &OpKind) -> Option<RecordedValue> {
        match kind {
            OpKind::Add { left, right } => {
                let (l, r) = self.extract_binary_numeric(left, right)?;
                match (l, r) {
                    (NumericValue::Int(a), NumericValue::Int(b)) => Some(RecordedValue::Int(a + b)),
                    (NumericValue::Float(a), NumericValue::Float(b)) => Some(RecordedValue::Float(a + b)),
                    (NumericValue::Int(a), NumericValue::Float(b)) => Some(RecordedValue::Float(a as f64 + b)),
                    (NumericValue::Float(a), NumericValue::Int(b)) => Some(RecordedValue::Float(a + b as f64)),
                }
            }
            OpKind::Sub { left, right } => {
                let (l, r) = self.extract_binary_numeric(left, right)?;
                match (l, r) {
                    (NumericValue::Int(a), NumericValue::Int(b)) => Some(RecordedValue::Int(a - b)),
                    (NumericValue::Float(a), NumericValue::Float(b)) => Some(RecordedValue::Float(a - b)),
                    (NumericValue::Int(a), NumericValue::Float(b)) => Some(RecordedValue::Float(a as f64 - b)),
                    (NumericValue::Float(a), NumericValue::Int(b)) => Some(RecordedValue::Float(a - b as f64)),
                }
            }
            OpKind::Mul { left, right } => {
                let (l, r) = self.extract_binary_numeric(left, right)?;
                match (l, r) {
                    (NumericValue::Int(a), NumericValue::Int(b)) => Some(RecordedValue::Int(a * b)),
                    (NumericValue::Float(a), NumericValue::Float(b)) => Some(RecordedValue::Float(a * b)),
                    (NumericValue::Int(a), NumericValue::Float(b)) => Some(RecordedValue::Float(a as f64 * b)),
                    (NumericValue::Float(a), NumericValue::Int(b)) => Some(RecordedValue::Float(a * b as f64)),
                }
            }
            OpKind::Div { left, right } => {
                let (l, r) = self.extract_binary_numeric(left, right)?;
                match (l, r) {
                    (NumericValue::Int(a), NumericValue::Int(b)) if b != 0 => {
                        Some(RecordedValue::Float(a as f64 / b as f64))
                    }
                    (NumericValue::Float(a), NumericValue::Float(b)) if b != 0.0 => {
                        Some(RecordedValue::Float(a / b))
                    }
                    (NumericValue::Int(a), NumericValue::Float(b)) if b != 0.0 => {
                        Some(RecordedValue::Float(a as f64 / b))
                    }
                    (NumericValue::Float(a), NumericValue::Int(b)) if b != 0 => {
                        Some(RecordedValue::Float(a / b as f64))
                    }
                    _ => None,
                }
            }
            OpKind::FloorDiv { left, right } => {
                let (l, r) = self.extract_binary_numeric(left, right)?;
                match (l, r) {
                    (NumericValue::Int(a), NumericValue::Int(b)) if b != 0 => {
                        Some(RecordedValue::Int(a / b))
                    }
                    _ => None,
                }
            }
            OpKind::Mod { left, right } => {
                let (l, r) = self.extract_binary_numeric(left, right)?;
                match (l, r) {
                    (NumericValue::Int(a), NumericValue::Int(b)) if b != 0 => {
                        Some(RecordedValue::Int(a % b))
                    }
                    _ => None,
                }
            }
            OpKind::Neg { value } => {
                let val = self.extract_literal(value)?;
                match val {
                    RecordedValue::Int(n) => Some(RecordedValue::Int(-n)),
                    RecordedValue::Float(f) => Some(RecordedValue::Float(-f)),
                    _ => None,
                }
            }
            OpKind::Abs { value } => {
                let val = self.extract_literal(value)?;
                match val {
                    RecordedValue::Int(n) => Some(RecordedValue::Int(n.abs())),
                    RecordedValue::Float(f) => Some(RecordedValue::Float(f.abs())),
                    _ => None,
                }
            }
            OpKind::Eq { left, right } => {
                let l = self.extract_literal(left)?;
                let r = self.extract_literal(right)?;
                Some(RecordedValue::Bool(l == r))
            }
            OpKind::Ne { left, right } => {
                let l = self.extract_literal(left)?;
                let r = self.extract_literal(right)?;
                Some(RecordedValue::Bool(l != r))
            }
            OpKind::Lt { left, right } => {
                let (l, r) = self.extract_binary_numeric(left, right)?;
                let result = match (l, r) {
                    (NumericValue::Int(a), NumericValue::Int(b)) => a < b,
                    (NumericValue::Float(a), NumericValue::Float(b)) => a < b,
                    (NumericValue::Int(a), NumericValue::Float(b)) => (a as f64) < b,
                    (NumericValue::Float(a), NumericValue::Int(b)) => a < (b as f64),
                };
                Some(RecordedValue::Bool(result))
            }
            OpKind::Le { left, right } => {
                let (l, r) = self.extract_binary_numeric(left, right)?;
                let result = match (l, r) {
                    (NumericValue::Int(a), NumericValue::Int(b)) => a <= b,
                    (NumericValue::Float(a), NumericValue::Float(b)) => a <= b,
                    (NumericValue::Int(a), NumericValue::Float(b)) => (a as f64) <= b,
                    (NumericValue::Float(a), NumericValue::Int(b)) => a <= (b as f64),
                };
                Some(RecordedValue::Bool(result))
            }
            OpKind::Gt { left, right } => {
                let (l, r) = self.extract_binary_numeric(left, right)?;
                let result = match (l, r) {
                    (NumericValue::Int(a), NumericValue::Int(b)) => a > b,
                    (NumericValue::Float(a), NumericValue::Float(b)) => a > b,
                    (NumericValue::Int(a), NumericValue::Float(b)) => (a as f64) > b,
                    (NumericValue::Float(a), NumericValue::Int(b)) => a > (b as f64),
                };
                Some(RecordedValue::Bool(result))
            }
            OpKind::Ge { left, right } => {
                let (l, r) = self.extract_binary_numeric(left, right)?;
                let result = match (l, r) {
                    (NumericValue::Int(a), NumericValue::Int(b)) => a >= b,
                    (NumericValue::Float(a), NumericValue::Float(b)) => a >= b,
                    (NumericValue::Int(a), NumericValue::Float(b)) => (a as f64) >= b,
                    (NumericValue::Float(a), NumericValue::Int(b)) => a >= (b as f64),
                };
                Some(RecordedValue::Bool(result))
            }
            OpKind::Not { value } => {
                let val = self.extract_literal(value)?;
                match val {
                    RecordedValue::Bool(b) => Some(RecordedValue::Bool(!b)),
                    _ => None,
                }
            }
            OpKind::Concat { left, right } => {
                let l = self.extract_literal(left)?;
                let r = self.extract_literal(right)?;
                match (l, r) {
                    (RecordedValue::String(a), RecordedValue::String(b)) => {
                        Some(RecordedValue::String(format!("{}{}", a, b)))
                    }
                    (RecordedValue::List(mut a), RecordedValue::List(b)) => {
                        a.extend(b);
                        Some(RecordedValue::List(a))
                    }
                    _ => None,
                }
            }
            OpKind::Len { value } => {
                let val = self.extract_literal(value)?;
                match val {
                    RecordedValue::String(s) => Some(RecordedValue::Int(s.len() as i64)),
                    RecordedValue::List(l) => Some(RecordedValue::Int(l.len() as i64)),
                    RecordedValue::Dict(d) => Some(RecordedValue::Int(d.len() as i64)),
                    RecordedValue::Bytes(b) => Some(RecordedValue::Int(b.len() as i64)),
                    _ => None,
                }
            }
            OpKind::Contains { haystack, needle } => {
                let h = self.extract_literal(haystack)?;
                let n = self.extract_literal(needle)?;
                match (h, n) {
                    (RecordedValue::String(s), RecordedValue::String(sub)) => {
                        Some(RecordedValue::Bool(s.contains(&sub)))
                    }
                    (RecordedValue::List(l), val) => {
                        Some(RecordedValue::Bool(l.contains(&val)))
                    }
                    (RecordedValue::Dict(d), RecordedValue::String(key)) => {
                        Some(RecordedValue::Bool(d.contains_key(&key)))
                    }
                    _ => None,
                }
            }
            OpKind::ToBool { value } => {
                let val = self.extract_literal(value)?;
                let b = match val {
                    RecordedValue::None => false,
                    RecordedValue::Bool(b) => b,
                    RecordedValue::Int(n) => n != 0,
                    RecordedValue::Float(f) => f != 0.0,
                    RecordedValue::String(s) => !s.is_empty(),
                    RecordedValue::Bytes(b) => !b.is_empty(),
                    RecordedValue::List(l) => !l.is_empty(),
                    RecordedValue::Dict(d) => !d.is_empty(),
                };
                Some(RecordedValue::Bool(b))
            }
            OpKind::ToInt { value } => {
                let val = self.extract_literal(value)?;
                match val {
                    RecordedValue::Int(n) => Some(RecordedValue::Int(n)),
                    RecordedValue::Float(f) => Some(RecordedValue::Int(f as i64)),
                    RecordedValue::Bool(b) => Some(RecordedValue::Int(if b { 1 } else { 0 })),
                    RecordedValue::String(s) => s.parse::<i64>().ok().map(RecordedValue::Int),
                    _ => None,
                }
            }
            OpKind::ToFloat { value } => {
                let val = self.extract_literal(value)?;
                match val {
                    RecordedValue::Int(n) => Some(RecordedValue::Float(n as f64)),
                    RecordedValue::Float(f) => Some(RecordedValue::Float(f)),
                    RecordedValue::Bool(b) => Some(RecordedValue::Float(if b { 1.0 } else { 0.0 })),
                    RecordedValue::String(s) => s.parse::<f64>().ok().map(RecordedValue::Float),
                    _ => None,
                }
            }
            OpKind::ToStr { value } => {
                let val = self.extract_literal(value)?;
                Some(RecordedValue::String(format!("{}", val)))
            }
            OpKind::JsonEncode { value } => {
                let val = self.extract_literal(value)?;
                serde_json::to_string(&val)
                    .ok()
                    .map(RecordedValue::String)
            }
            OpKind::JsonDecode { string } => {
                let val = self.extract_literal(string)?;
                if let RecordedValue::String(s) = val {
                    serde_json::from_str::<RecordedValue>(&s).ok()
                } else {
                    None
                }
            }
            OpKind::If { condition, then_value, else_value } => {
                let cond = self.extract_literal(condition)?;
                let is_true = match cond {
                    RecordedValue::Bool(b) => b,
                    _ => return None,
                };
                if is_true {
                    self.extract_literal(then_value)
                } else {
                    self.extract_literal(else_value)
                }
            }
            OpKind::Index { base, index } => {
                let base_val = self.extract_literal(base)?;
                let idx_val = self.extract_literal(index)?;
                match (base_val, idx_val) {
                    (RecordedValue::List(l), RecordedValue::Int(i)) => {
                        let idx = if i < 0 {
                            (l.len() as i64 + i) as usize
                        } else {
                            i as usize
                        };
                        l.get(idx).cloned()
                    }
                    (RecordedValue::Dict(d), RecordedValue::String(k)) => {
                        d.get(&k).cloned()
                    }
                    (RecordedValue::String(s), RecordedValue::Int(i)) => {
                        let chars: Vec<char> = s.chars().collect();
                        let idx = if i < 0 {
                            (chars.len() as i64 + i) as usize
                        } else {
                            i as usize
                        };
                        chars.get(idx).map(|c| RecordedValue::String(c.to_string()))
                    }
                    _ => None,
                }
            }
            OpKind::Min { values } => {
                let val = self.extract_literal(values)?;
                if let RecordedValue::List(items) = val {
                    self.find_min(&items)
                } else {
                    None
                }
            }
            OpKind::Max { values } => {
                let val = self.extract_literal(values)?;
                if let RecordedValue::List(items) = val {
                    self.find_max(&items)
                } else {
                    None
                }
            }
            OpKind::Sum { values, start } => {
                let val = self.extract_literal(values)?;
                let start_val = self.extract_literal(start)?;
                if let (RecordedValue::List(items), RecordedValue::Int(s)) = (val, start_val) {
                    let mut sum = s;
                    for item in items {
                        match item {
                            RecordedValue::Int(n) => sum += n,
                            _ => return None,
                        }
                    }
                    Some(RecordedValue::Int(sum))
                } else {
                    None
                }
            }
            OpKind::Sorted { values } => {
                let val = self.extract_literal(values)?;
                if let RecordedValue::List(mut items) = val {
                    if items.iter().all(|v| matches!(v, RecordedValue::Int(_))) {
                        items.sort_by(|a, b| {
                            if let (RecordedValue::Int(x), RecordedValue::Int(y)) = (a, b) {
                                x.cmp(y)
                            } else {
                                std::cmp::Ordering::Equal
                            }
                        });
                        Some(RecordedValue::List(items))
                    } else if items.iter().all(|v| matches!(v, RecordedValue::String(_))) {
                        items.sort_by(|a, b| {
                            if let (RecordedValue::String(x), RecordedValue::String(y)) = (a, b) {
                                x.cmp(y)
                            } else {
                                std::cmp::Ordering::Equal
                            }
                        });
                        Some(RecordedValue::List(items))
                    } else {
                        None
                    }
                } else {
                    None
                }
            }
            OpKind::Reversed { values } => {
                let val = self.extract_literal(values)?;
                if let RecordedValue::List(mut items) = val {
                    items.reverse();
                    Some(RecordedValue::List(items))
                } else if let RecordedValue::String(s) = val {
                    Some(RecordedValue::String(s.chars().rev().collect()))
                } else {
                    None
                }
            }
            _ => None,
        }
    }

    fn extract_literal(&self, value_ref: &ValueRef) -> Option<RecordedValue> {
        if let ValueRef::Literal(v) = value_ref {
            Some(v.clone())
        } else {
            None
        }
    }

    fn extract_binary_numeric(
        &self,
        left: &ValueRef,
        right: &ValueRef,
    ) -> Option<(NumericValue, NumericValue)> {
        let l = self.extract_literal(left)?;
        let r = self.extract_literal(right)?;
        let l_num = match l {
            RecordedValue::Int(n) => NumericValue::Int(n),
            RecordedValue::Float(f) => NumericValue::Float(f),
            _ => return None,
        };
        let r_num = match r {
            RecordedValue::Int(n) => NumericValue::Int(n),
            RecordedValue::Float(f) => NumericValue::Float(f),
            _ => return None,
        };
        Some((l_num, r_num))
    }

    fn find_min(&self, items: &[RecordedValue]) -> Option<RecordedValue> {
        if items.is_empty() {
            return None;
        }
        if items.iter().all(|v| matches!(v, RecordedValue::Int(_))) {
            items
                .iter()
                .filter_map(|v| if let RecordedValue::Int(n) = v { Some(*n) } else { None })
                .min()
                .map(RecordedValue::Int)
        } else if items.iter().all(|v| matches!(v, RecordedValue::Float(_))) {
            items
                .iter()
                .filter_map(|v| if let RecordedValue::Float(f) = v { Some(*f) } else { None })
                .fold(None, |min, f| match min {
                    None => Some(f),
                    Some(m) if f < m => Some(f),
                    _ => min,
                })
                .map(RecordedValue::Float)
        } else {
            None
        }
    }

    fn find_max(&self, items: &[RecordedValue]) -> Option<RecordedValue> {
        if items.is_empty() {
            return None;
        }
        if items.iter().all(|v| matches!(v, RecordedValue::Int(_))) {
            items
                .iter()
                .filter_map(|v| if let RecordedValue::Int(n) = v { Some(*n) } else { None })
                .max()
                .map(RecordedValue::Int)
        } else if items.iter().all(|v| matches!(v, RecordedValue::Float(_))) {
            items
                .iter()
                .filter_map(|v| if let RecordedValue::Float(f) = v { Some(*f) } else { None })
                .fold(None, |max, f| match max {
                    None => Some(f),
                    Some(m) if f > m => Some(f),
                    _ => max,
                })
                .map(RecordedValue::Float)
        } else {
            None
        }
    }

    fn remove_folded_ops(&self, plan: &mut Plan, folded: &HashMap<OpId, RecordedValue>) {
        plan.remove_ops(|op| folded.contains_key(&op.id));
    }

    fn dead_code_eliminate(&self, mut plan: Plan) -> Plan {
        use std::collections::HashSet;

        let mut used_ops: HashSet<OpId> = HashSet::new();

        for op in plan.ops() {
            if self.has_side_effects(&op.kind) {
                used_ops.insert(op.id);
            }
        }

        loop {
            let mut changed = false;
            for op in plan.ops() {
                if used_ops.contains(&op.id) {
                    for input in &op.inputs {
                        if used_ops.insert(*input) {
                            changed = true;
                        }
                    }
                    for value_ref in op.kind.collect_value_refs() {
                        if let Some(op_id) = value_ref.referenced_op() {
                            if used_ops.insert(op_id) {
                                changed = true;
                            }
                        }
                    }
                }
            }
            if !changed {
                break;
            }
        }

        plan.remove_ops(|op| !used_ops.contains(&op.id));
        plan
    }

    fn has_side_effects(&self, kind: &OpKind) -> bool {
        matches!(
            kind,
            OpKind::ReadFile { .. }
            | OpKind::WriteFile { .. }
            | OpKind::AppendFile { .. }
            | OpKind::DeleteFile { .. }
            | OpKind::ListDir { .. }
            | OpKind::Mkdir { .. }
            | OpKind::Rmdir { .. }
            | OpKind::CopyFile { .. }
            | OpKind::MoveFile { .. }
            | OpKind::FileExists { .. }
            | OpKind::IsDir { .. }
            | OpKind::IsFile { .. }
            | OpKind::FileSize { .. }
            | OpKind::HttpRequest { .. }
            | OpKind::TcpConnect { .. }
            | OpKind::TcpSend { .. }
            | OpKind::TcpRecv { .. }
            | OpKind::TcpClose { .. }
            | OpKind::TcpListen { .. }
            | OpKind::TcpAccept { .. }
            | OpKind::UdpBind { .. }
            | OpKind::UdpSendTo { .. }
            | OpKind::UdpRecvFrom { .. }
            | OpKind::UdpClose { .. }
            | OpKind::UnixConnect { .. }
            | OpKind::UnixSend { .. }
            | OpKind::UnixRecv { .. }
            | OpKind::UnixClose { .. }
            | OpKind::UnixListen { .. }
            | OpKind::UnixAccept { .. }
            | OpKind::Exec { .. }
            | OpKind::EnvGet { .. }
            | OpKind::Sleep { .. }
            | OpKind::Now
            | OpKind::Print { .. }
        )
    }
}

enum NumericValue {
    Int(i64),
    Float(f64),
}

#[cfg(test)]
mod tests {
    use super::*;
    use blueprint_common::ValueRef;

    #[test]
    fn test_constant_fold_add() {
        let mut plan = Plan::new();
        plan.add_op(
            OpKind::Add {
                left: ValueRef::literal_int(1),
                right: ValueRef::literal_int(2),
            },
            None,
        );
        plan.add_op(
            OpKind::Print {
                message: ValueRef::op_output(OpId(0)),
            },
            None,
        );

        let optimizer = PlanOptimizer::new(OptLevel::Basic);
        let optimized = optimizer.optimize(plan);

        assert_eq!(optimized.len(), 1);
        let print_op = optimized.get_op(OpId(1)).unwrap();
        if let OpKind::Print { message } = &print_op.kind {
            assert!(matches!(message, ValueRef::Literal(RecordedValue::Int(3))));
        } else {
            panic!("Expected Print op");
        }
    }

    #[test]
    fn test_constant_fold_chain() {
        let mut plan = Plan::new();
        let a = plan.add_op(
            OpKind::Add {
                left: ValueRef::literal_int(1),
                right: ValueRef::literal_int(1),
            },
            None,
        );
        plan.add_op(
            OpKind::Mul {
                left: ValueRef::op_output(a),
                right: ValueRef::literal_int(3),
            },
            None,
        );
        plan.add_op(
            OpKind::Print {
                message: ValueRef::op_output(OpId(1)),
            },
            None,
        );

        let optimizer = PlanOptimizer::new(OptLevel::Basic);
        let optimized = optimizer.optimize(plan);

        assert_eq!(optimized.len(), 1);
        let print_op = optimized.get_op(OpId(2)).unwrap();
        if let OpKind::Print { message } = &print_op.kind {
            assert!(matches!(message, ValueRef::Literal(RecordedValue::Int(6))));
        } else {
            panic!("Expected Print op");
        }
    }

    #[test]
    fn test_no_fold_io_dependency() {
        let mut plan = Plan::new();
        let read = plan.add_op(
            OpKind::ReadFile {
                path: ValueRef::literal_string("config.txt"),
            },
            None,
        );
        plan.add_op(
            OpKind::Len {
                value: ValueRef::op_output(read),
            },
            None,
        );

        let optimizer = PlanOptimizer::new(OptLevel::Basic);
        let optimized = optimizer.optimize(plan);

        assert_eq!(optimized.len(), 2);
    }

    #[test]
    fn test_dead_code_elimination() {
        let mut plan = Plan::new();
        plan.add_op(
            OpKind::Add {
                left: ValueRef::literal_int(1),
                right: ValueRef::literal_int(2),
            },
            None,
        );
        plan.add_op(
            OpKind::Print {
                message: ValueRef::literal_string("hello"),
            },
            None,
        );

        let optimizer = PlanOptimizer::new(OptLevel::Aggressive);
        let optimized = optimizer.optimize(plan);

        assert_eq!(optimized.len(), 1);
        assert!(matches!(
            optimized.get_op(OpId(1)).unwrap().kind,
            OpKind::Print { .. }
        ));
    }
}
