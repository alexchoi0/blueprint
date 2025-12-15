use std::collections::HashSet;

use blueprint_approval::Policy;
use blueprint_common::{OpId, OpKind, ValueRef, Plan, CycleError};

#[derive(Debug)]
pub struct ValidationResult {
    pub errors: Vec<ValidationError>,
    pub warnings: Vec<ValidationWarning>,
    pub levels: Option<Vec<Vec<OpId>>>,
}

impl ValidationResult {
    pub fn is_valid(&self) -> bool {
        self.errors.is_empty()
    }

    pub fn has_warnings(&self) -> bool {
        !self.warnings.is_empty()
    }
}

#[derive(Debug, Clone)]
pub enum ValidationError {
    CycleDetected { ops: Vec<OpId> },
    UnknownOpReference { from: OpId, to: OpId },
    InvalidCombinatorCount { op: OpId, count: usize, available: usize },
    PolicyDenied { op: OpId, reason: String },
    MalformedUrl { op: OpId, url: String },
    MalformedPath { op: OpId, path: String },
    UnsupportedPlatform { op: OpId, operation: String, platform: String },
}

impl std::fmt::Display for ValidationError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ValidationError::CycleDetected { ops } => {
                write!(f, "Cycle detected involving ops: {:?}", ops.iter().map(|o| o.0).collect::<Vec<_>>())
            }
            ValidationError::UnknownOpReference { from, to } => {
                write!(f, "Op {} references unknown op {}", from.0, to.0)
            }
            ValidationError::InvalidCombinatorCount { op, count, available } => {
                write!(f, "Op {} requires {} successes but only {} ops available", op.0, count, available)
            }
            ValidationError::PolicyDenied { op, reason } => {
                write!(f, "Op {} denied by policy: {}", op.0, reason)
            }
            ValidationError::MalformedUrl { op, url } => {
                write!(f, "Op {} has malformed URL: {}", op.0, url)
            }
            ValidationError::MalformedPath { op, path } => {
                write!(f, "Op {} has malformed path: {}", op.0, path)
            }
            ValidationError::UnsupportedPlatform { op, operation, platform } => {
                write!(f, "Op {} uses {} which is not supported on {}", op.0, operation, platform)
            }
        }
    }
}

impl std::error::Error for ValidationError {}

#[derive(Debug, Clone)]
pub enum ValidationWarning {
    UnusedOp { op: OpId },
    PotentialRaceCondition { ops: Vec<OpId>, resource: String },
    DynamicValueNeedsRuntimeApproval { op: OpId },
    LargePlan { op_count: usize },
}

impl std::fmt::Display for ValidationWarning {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ValidationWarning::UnusedOp { op } => {
                write!(f, "Op {} result is never used", op.0)
            }
            ValidationWarning::PotentialRaceCondition { ops, resource } => {
                write!(
                    f,
                    "Potential race condition on '{}' involving ops: {:?}",
                    resource,
                    ops.iter().map(|o| o.0).collect::<Vec<_>>()
                )
            }
            ValidationWarning::DynamicValueNeedsRuntimeApproval { op } => {
                write!(f, "Op {} has dynamic values requiring runtime approval", op.0)
            }
            ValidationWarning::LargePlan { op_count } => {
                write!(f, "Plan has {} ops, which may be slow", op_count)
            }
        }
    }
}

pub struct PlanValidator;

impl PlanValidator {
    pub fn validate(plan: &Plan, policy: Option<&Policy>) -> ValidationResult {
        let mut errors = Vec::new();
        let mut warnings = Vec::new();

        errors.extend(Self::check_references(plan));

        let levels = match plan.compute_levels() {
            Ok(levels) => Some(levels),
            Err(CycleError { ops }) => {
                errors.push(ValidationError::CycleDetected { ops });
                None
            }
        };

        errors.extend(Self::check_combinators(plan));
        errors.extend(Self::check_platform_support(plan));

        if let Some(policy) = policy {
            errors.extend(Self::check_policy(plan, policy));
        }

        warnings.extend(Self::check_unused_ops(plan));
        warnings.extend(Self::check_race_conditions(plan, &levels));
        warnings.extend(Self::check_dynamic_values(plan));

        if plan.len() > 1000 {
            warnings.push(ValidationWarning::LargePlan { op_count: plan.len() });
        }

        ValidationResult {
            errors,
            warnings,
            levels,
        }
    }

    fn check_references(plan: &Plan) -> Vec<ValidationError> {
        let mut errors = Vec::new();
        let valid_ids: HashSet<OpId> = plan.ops().map(|op| op.id).collect();

        for op in plan.ops() {
            for &input in &op.inputs {
                if !valid_ids.contains(&input) {
                    errors.push(ValidationError::UnknownOpReference {
                        from: op.id,
                        to: input,
                    });
                }
            }

            for value_ref in op.kind.collect_value_refs() {
                if let Some(ref_id) = value_ref.referenced_op() {
                    if !valid_ids.contains(&ref_id) {
                        errors.push(ValidationError::UnknownOpReference {
                            from: op.id,
                            to: ref_id,
                        });
                    }
                }
            }
        }

        errors
    }

    fn check_combinators(plan: &Plan) -> Vec<ValidationError> {
        let mut errors = Vec::new();

        for op in plan.ops() {
            match &op.kind {
                OpKind::AtLeast { ops, count } => {
                    if *count > ops.len() {
                        errors.push(ValidationError::InvalidCombinatorCount {
                            op: op.id,
                            count: *count,
                            available: ops.len(),
                        });
                    }
                }
                OpKind::AtMost { ops, count } => {
                    if *count >= ops.len() {
                    }
                }
                _ => {}
            }
        }

        errors
    }

    fn check_platform_support(plan: &Plan) -> Vec<ValidationError> {
        let mut errors = Vec::new();

        for op in plan.ops() {
            match &op.kind {
                OpKind::UnixConnect { .. }
                | OpKind::UnixSend { .. }
                | OpKind::UnixRecv { .. }
                | OpKind::UnixClose { .. }
                | OpKind::UnixListen { .. }
                | OpKind::UnixAccept { .. } => {
                    if !cfg!(unix) {
                        errors.push(ValidationError::UnsupportedPlatform {
                            op: op.id,
                            operation: "Unix sockets".to_string(),
                            platform: std::env::consts::OS.to_string(),
                        });
                    }
                }
                _ => {}
            }
        }

        errors
    }

    fn check_policy(plan: &Plan, policy: &Policy) -> Vec<ValidationError> {
        let mut errors = Vec::new();

        for op in plan.ops() {
            if !op.kind.requires_approval() {
                continue;
            }

            if let Some(action) = Self::op_kind_to_action(&op.kind) {
                use blueprint_approval::PolicyDecision;
                if let PolicyDecision::Deny = policy.check(&action) {
                    errors.push(ValidationError::PolicyDenied {
                        op: op.id,
                        reason: format!("{}", action),
                    });
                }
            }
        }

        errors
    }

    fn op_kind_to_action(kind: &OpKind) -> Option<blueprint_approval::Action> {
        use blueprint_approval::Action;

        match kind {
            OpKind::ReadFile { path } => {
                if let ValueRef::Literal(v) = path {
                    v.as_string().map(|s| Action::ReadFile { path: s.to_string() })
                } else {
                    None
                }
            }
            OpKind::WriteFile { path, .. } => {
                if let ValueRef::Literal(v) = path {
                    v.as_string().map(|s| Action::WriteFile { path: s.to_string() })
                } else {
                    None
                }
            }
            OpKind::AppendFile { path, .. } => {
                if let ValueRef::Literal(v) = path {
                    v.as_string().map(|s| Action::AppendFile { path: s.to_string() })
                } else {
                    None
                }
            }
            OpKind::DeleteFile { path } => {
                if let ValueRef::Literal(v) = path {
                    v.as_string().map(|s| Action::DeleteFile { path: s.to_string() })
                } else {
                    None
                }
            }
            OpKind::ListDir { path } => {
                if let ValueRef::Literal(v) = path {
                    v.as_string().map(|s| Action::ListDir { path: s.to_string() })
                } else {
                    None
                }
            }
            OpKind::Mkdir { path, .. } => {
                if let ValueRef::Literal(v) = path {
                    v.as_string().map(|s| Action::CreateDir { path: s.to_string() })
                } else {
                    None
                }
            }
            OpKind::Rmdir { path, .. } => {
                if let ValueRef::Literal(v) = path {
                    v.as_string().map(|s| Action::DeleteDir { path: s.to_string() })
                } else {
                    None
                }
            }
            OpKind::CopyFile { src, dst } => {
                match (src, dst) {
                    (ValueRef::Literal(s), ValueRef::Literal(d)) => {
                        match (s.as_string(), d.as_string()) {
                            (Some(src), Some(dst)) => Some(Action::CopyFile {
                                src: src.to_string(),
                                dst: dst.to_string(),
                            }),
                            _ => None,
                        }
                    }
                    _ => None,
                }
            }
            OpKind::MoveFile { src, dst } => {
                match (src, dst) {
                    (ValueRef::Literal(s), ValueRef::Literal(d)) => {
                        match (s.as_string(), d.as_string()) {
                            (Some(src), Some(dst)) => Some(Action::MoveFile {
                                src: src.to_string(),
                                dst: dst.to_string(),
                            }),
                            _ => None,
                        }
                    }
                    _ => None,
                }
            }
            OpKind::HttpRequest { method, url, .. } => {
                match (method, url) {
                    (ValueRef::Literal(m), ValueRef::Literal(u)) => {
                        match (m.as_string(), u.as_string()) {
                            (Some(method), Some(url)) => Some(Action::HttpRequest {
                                method: method.to_string(),
                                url: url.to_string(),
                            }),
                            _ => None,
                        }
                    }
                    _ => None,
                }
            }
            OpKind::TcpConnect { host, port } => {
                match (host, port) {
                    (ValueRef::Literal(h), ValueRef::Literal(p)) => {
                        match (h.as_string(), p.as_int()) {
                            (Some(host), Some(port)) => Some(Action::TcpConnect {
                                host: host.to_string(),
                                port: port as u16,
                            }),
                            _ => None,
                        }
                    }
                    _ => None,
                }
            }
            OpKind::TcpListen { host, port } => {
                match (host, port) {
                    (ValueRef::Literal(h), ValueRef::Literal(p)) => {
                        match (h.as_string(), p.as_int()) {
                            (Some(host), Some(port)) => Some(Action::TcpListen {
                                host: host.to_string(),
                                port: port as u16,
                            }),
                            _ => None,
                        }
                    }
                    _ => None,
                }
            }
            OpKind::UdpBind { host, port } => {
                match (host, port) {
                    (ValueRef::Literal(h), ValueRef::Literal(p)) => {
                        match (h.as_string(), p.as_int()) {
                            (Some(host), Some(port)) => Some(Action::UdpBind {
                                host: host.to_string(),
                                port: port as u16,
                            }),
                            _ => None,
                        }
                    }
                    _ => None,
                }
            }
            OpKind::UdpSendTo { host, port, .. } => {
                match (host, port) {
                    (ValueRef::Literal(h), ValueRef::Literal(p)) => {
                        match (h.as_string(), p.as_int()) {
                            (Some(host), Some(port)) => Some(Action::UdpSendTo {
                                host: host.to_string(),
                                port: port as u16,
                            }),
                            _ => None,
                        }
                    }
                    _ => None,
                }
            }
            OpKind::Exec { command, args } => {
                if let ValueRef::Literal(cmd) = command {
                    cmd.as_string().map(|c| {
                        let args_vec = if let ValueRef::Literal(a) = args {
                            a.as_list()
                                .map(|l| {
                                    l.iter()
                                        .filter_map(|v| v.as_string().map(|s| s.to_string()))
                                        .collect()
                                })
                                .unwrap_or_default()
                        } else {
                            Vec::new()
                        };
                        Action::Exec {
                            command: c.to_string(),
                            args: args_vec,
                        }
                    })
                } else {
                    None
                }
            }
            _ => None,
        }
    }

    fn check_unused_ops(plan: &Plan) -> Vec<ValidationWarning> {
        let mut warnings = Vec::new();
        let mut used_ops: HashSet<OpId> = HashSet::new();

        for op in plan.ops() {
            for &input in &op.inputs {
                used_ops.insert(input);
            }
        }

        for op in plan.ops() {
            if !used_ops.contains(&op.id) {
                match &op.kind {
                    OpKind::WriteFile { .. }
                    | OpKind::AppendFile { .. }
                    | OpKind::DeleteFile { .. }
                    | OpKind::Mkdir { .. }
                    | OpKind::Rmdir { .. }
                    | OpKind::CopyFile { .. }
                    | OpKind::MoveFile { .. }
                    | OpKind::TcpClose { .. }
                    | OpKind::UdpClose { .. }
                    | OpKind::Print { .. }
                    | OpKind::Sleep { .. } => {}
                    _ => {
                        if plan.ops().last().map(|o| o.id) != Some(op.id) {
                            warnings.push(ValidationWarning::UnusedOp { op: op.id });
                        }
                    }
                }
            }
        }

        warnings
    }

    fn check_race_conditions(plan: &Plan, levels: &Option<Vec<Vec<OpId>>>) -> Vec<ValidationWarning> {
        let mut warnings = Vec::new();

        if let Some(levels) = levels {
            for level in levels {
                if level.len() > 1 {
                    let mut write_paths: Vec<(OpId, String)> = Vec::new();

                    for &op_id in level {
                        if let Some(op) = plan.get_op(op_id) {
                            let path = match &op.kind {
                                OpKind::WriteFile { path, .. }
                                | OpKind::AppendFile { path, .. }
                                | OpKind::DeleteFile { path }
                                | OpKind::Mkdir { path, .. }
                                | OpKind::Rmdir { path, .. } => {
                                    if let ValueRef::Literal(v) = path {
                                        v.as_string().map(|s| s.to_string())
                                    } else {
                                        None
                                    }
                                }
                                OpKind::CopyFile { dst, .. } | OpKind::MoveFile { dst, .. } => {
                                    if let ValueRef::Literal(v) = dst {
                                        v.as_string().map(|s| s.to_string())
                                    } else {
                                        None
                                    }
                                }
                                _ => None,
                            };

                            if let Some(path) = path {
                                write_paths.push((op_id, path));
                            }
                        }
                    }

                    for i in 0..write_paths.len() {
                        for j in (i + 1)..write_paths.len() {
                            if write_paths[i].1 == write_paths[j].1 {
                                warnings.push(ValidationWarning::PotentialRaceCondition {
                                    ops: vec![write_paths[i].0, write_paths[j].0],
                                    resource: write_paths[i].1.clone(),
                                });
                            }
                        }
                    }
                }
            }
        }

        warnings
    }

    fn check_dynamic_values(plan: &Plan) -> Vec<ValidationWarning> {
        let mut warnings = Vec::new();

        for op in plan.ops() {
            if !op.kind.requires_approval() {
                continue;
            }

            let has_dynamic = op.kind.collect_value_refs().iter().any(|v| v.is_dynamic());

            if has_dynamic {
                warnings.push(ValidationWarning::DynamicValueNeedsRuntimeApproval { op: op.id });
            }
        }

        warnings
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_valid_plan() {
        let mut plan = Plan::new();
        plan.add_op(OpKind::Now, None);

        let result = PlanValidator::validate(&plan, None);
        assert!(result.is_valid());
    }

    #[test]
    fn test_invalid_reference() {
        let mut plan = Plan::new();
        plan.add_op(
            OpKind::JsonDecode {
                string: ValueRef::op_output(OpId(999)),
            },
            None,
        );

        let result = PlanValidator::validate(&plan, None);
        assert!(!result.is_valid());
        assert!(matches!(
            result.errors.first(),
            Some(ValidationError::UnknownOpReference { .. })
        ));
    }

    #[test]
    fn test_invalid_at_least_count() {
        let mut plan = Plan::new();
        let op1 = plan.add_op(OpKind::Now, None);
        let op2 = plan.add_op(OpKind::Now, None);

        plan.add_op(
            OpKind::AtLeast {
                ops: vec![op1, op2],
                count: 5,
            },
            None,
        );

        let result = PlanValidator::validate(&plan, None);
        assert!(!result.is_valid());
    }

    #[test]
    #[cfg(unix)]
    fn test_unix_sockets_valid_on_unix() {
        let mut plan = Plan::new();
        plan.add_op(
            OpKind::UnixConnect {
                path: ValueRef::literal_string("/tmp/test.sock"),
            },
            None,
        );

        let result = PlanValidator::validate(&plan, None);
        let platform_errors: Vec<_> = result
            .errors
            .iter()
            .filter(|e| matches!(e, ValidationError::UnsupportedPlatform { .. }))
            .collect();
        assert!(platform_errors.is_empty(), "Unix sockets should be valid on Unix");
    }

    #[test]
    #[cfg(not(unix))]
    fn test_unix_sockets_invalid_on_windows() {
        let mut plan = Plan::new();
        plan.add_op(
            OpKind::UnixConnect {
                path: ValueRef::literal_string("/tmp/test.sock"),
            },
            None,
        );

        let result = PlanValidator::validate(&plan, None);
        assert!(!result.is_valid());
        assert!(matches!(
            result.errors.first(),
            Some(ValidationError::UnsupportedPlatform { operation, .. }) if operation == "Unix sockets"
        ));
    }

    #[test]
    fn test_check_platform_support_returns_errors_for_all_unix_ops() {
        let mut plan = Plan::new();
        plan.add_op(OpKind::UnixConnect { path: ValueRef::literal_string("/tmp/s") }, None);
        plan.add_op(OpKind::UnixListen { path: ValueRef::literal_string("/tmp/s") }, None);
        plan.add_op(OpKind::UnixSend { handle: ValueRef::literal_int(0), data: ValueRef::literal_string("d") }, None);
        plan.add_op(OpKind::UnixRecv { handle: ValueRef::literal_int(0), max_bytes: ValueRef::literal_int(1024) }, None);
        plan.add_op(OpKind::UnixClose { handle: ValueRef::literal_int(0) }, None);
        plan.add_op(OpKind::UnixAccept { listener: ValueRef::literal_int(0) }, None);

        let errors = PlanValidator::check_platform_support(&plan);

        if cfg!(unix) {
            assert!(errors.is_empty(), "No errors expected on Unix");
        } else {
            assert_eq!(errors.len(), 6, "All 6 Unix socket ops should fail on non-Unix");
            for error in &errors {
                assert!(matches!(error, ValidationError::UnsupportedPlatform { operation, .. } if operation == "Unix sockets"));
            }
        }
    }
}
