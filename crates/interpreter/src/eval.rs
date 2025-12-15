use anyhow::Result;
use blueprint_common::{Plan, RecordedValue};

use crate::BlueprintInterpreter;

pub fn eval_plan(plan: &Plan) -> Result<String> {
    if plan.ops().count() == 0 {
        return Ok("None".to_string());
    }

    let rt = tokio::runtime::Runtime::new()?;
    let result: Result<String> = rt.block_on(async {
        eval_plan_async(plan).await
    });

    result
}

pub async fn eval_plan_async(plan: &Plan) -> Result<String> {
    if plan.ops().count() == 0 {
        return Ok("None".to_string());
    }

    let mut interpreter = BlueprintInterpreter::new();
    let cache = interpreter.execute(plan).await
        .map_err(|e| anyhow::anyhow!("Execution error: {:?}", e))?;

    if let Some(last_op) = plan.ops().last() {
        if let Some(value) = cache.get_value(last_op.id) {
            return Ok(recorded_value_to_string(&value));
        }
    }
    Ok("None".to_string())
}

pub fn recorded_value_to_string(value: &RecordedValue) -> String {
    match value {
        RecordedValue::None => "None".to_string(),
        RecordedValue::Bool(b) => if *b { "True".to_string() } else { "False".to_string() },
        RecordedValue::Int(i) => i.to_string(),
        RecordedValue::Float(f) => {
            if f.fract() == 0.0 && f.abs() < 1e15 {
                format!("{}.0", *f as i64)
            } else {
                f.to_string()
            }
        }
        RecordedValue::String(s) => s.clone(),
        RecordedValue::Bytes(b) => {
            let escaped: String = b.iter()
                .map(|byte| {
                    if *byte >= 32 && *byte < 127 && *byte != b'"' && *byte != b'\\' {
                        (*byte as char).to_string()
                    } else {
                        format!("\\x{:02x}", byte)
                    }
                })
                .collect();
            format!("b\"{}\"", escaped)
        }
        RecordedValue::List(l) => {
            let items: Vec<String> = l.iter().map(recorded_value_to_repr).collect();
            format!("[{}]", items.join(", "))
        }
        RecordedValue::Dict(d) => {
            let items: Vec<String> = d.iter()
                .map(|(k, v)| format!("\"{}\": {}", k, recorded_value_to_repr(v)))
                .collect();
            format!("{{{}}}", items.join(", "))
        }
    }
}

pub fn recorded_value_to_repr(value: &RecordedValue) -> String {
    match value {
        RecordedValue::String(s) => format!("\"{}\"", s),
        _ => recorded_value_to_string(value),
    }
}
