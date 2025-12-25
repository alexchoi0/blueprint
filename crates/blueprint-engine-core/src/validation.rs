use crate::{BlueprintError, Result, Value};

pub fn require_args(name: &str, args: &[Value], count: usize) -> Result<()> {
    if args.len() != count {
        return Err(BlueprintError::ArgumentError {
            message: format!(
                "{}() takes exactly {} argument(s) ({} given)",
                name,
                count,
                args.len()
            ),
        });
    }
    Ok(())
}

pub fn require_args_min(name: &str, args: &[Value], min: usize) -> Result<()> {
    if args.len() < min {
        return Err(BlueprintError::ArgumentError {
            message: format!(
                "{}() requires at least {} argument(s) ({} given)",
                name,
                min,
                args.len()
            ),
        });
    }
    Ok(())
}

pub fn require_args_range(name: &str, args: &[Value], min: usize, max: usize) -> Result<()> {
    if args.len() < min || args.len() > max {
        return Err(BlueprintError::ArgumentError {
            message: format!(
                "{}() takes {}-{} argument(s) ({} given)",
                name,
                min,
                max,
                args.len()
            ),
        });
    }
    Ok(())
}

pub fn require_string(value: &Value) -> Result<String> {
    value.as_string().map_err(|_| BlueprintError::TypeError {
        expected: "string".into(),
        actual: value.type_name().into(),
    })
}

pub fn require_int(value: &Value) -> Result<i64> {
    value.as_int().map_err(|_| BlueprintError::TypeError {
        expected: "int".into(),
        actual: value.type_name().into(),
    })
}

pub fn require_float(value: &Value) -> Result<f64> {
    value.as_float().map_err(|_| BlueprintError::TypeError {
        expected: "float".into(),
        actual: value.type_name().into(),
    })
}

pub fn require_bool(value: &Value) -> Result<bool> {
    value.as_bool().map_err(|_| BlueprintError::TypeError {
        expected: "bool".into(),
        actual: value.type_name().into(),
    })
}

pub fn get_arg<'a>(name: &str, args: &'a [Value], index: usize) -> Result<&'a Value> {
    args.get(index)
        .ok_or_else(|| BlueprintError::ArgumentError {
            message: format!("{}() missing required argument at position {}", name, index),
        })
}

pub fn get_string_arg(name: &str, args: &[Value], index: usize) -> Result<String> {
    let value = get_arg(name, args, index)?;
    require_string(value)
}

pub fn get_int_arg(name: &str, args: &[Value], index: usize) -> Result<i64> {
    let value = get_arg(name, args, index)?;
    require_int(value)
}

pub fn get_float_arg(name: &str, args: &[Value], index: usize) -> Result<f64> {
    let value = get_arg(name, args, index)?;
    require_float(value)
}

pub fn get_bool_arg(name: &str, args: &[Value], index: usize) -> Result<bool> {
    let value = get_arg(name, args, index)?;
    require_bool(value)
}

pub fn get_optional_string_arg(args: &[Value], index: usize) -> Result<Option<String>> {
    match args.get(index) {
        Some(v) if !matches!(v, Value::None) => Ok(Some(v.as_string().map_err(|_| {
            BlueprintError::TypeError {
                expected: "string".into(),
                actual: v.type_name().into(),
            }
        })?)),
        _ => Ok(None),
    }
}

pub fn get_optional_int_arg(args: &[Value], index: usize) -> Result<Option<i64>> {
    match args.get(index) {
        Some(v) if !matches!(v, Value::None) => {
            Ok(Some(v.as_int().map_err(|_| BlueprintError::TypeError {
                expected: "int".into(),
                actual: v.type_name().into(),
            })?))
        }
        _ => Ok(None),
    }
}

pub fn get_optional_float_arg(args: &[Value], index: usize) -> Result<Option<f64>> {
    match args.get(index) {
        Some(v) if !matches!(v, Value::None) => Ok(Some(v.as_float().map_err(|_| {
            BlueprintError::TypeError {
                expected: "float".into(),
                actual: v.type_name().into(),
            }
        })?)),
        _ => Ok(None),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Arc;

    #[test]
    fn test_require_args_exact() {
        let args = vec![Value::Int(1), Value::Int(2)];
        assert!(require_args("test", &args, 2).is_ok());
        assert!(require_args("test", &args, 1).is_err());
        assert!(require_args("test", &args, 3).is_err());
    }

    #[test]
    fn test_require_args_min() {
        let args = vec![Value::Int(1), Value::Int(2)];
        assert!(require_args_min("test", &args, 1).is_ok());
        assert!(require_args_min("test", &args, 2).is_ok());
        assert!(require_args_min("test", &args, 3).is_err());
    }

    #[test]
    fn test_require_args_range() {
        let args = vec![Value::Int(1), Value::Int(2)];
        assert!(require_args_range("test", &args, 1, 3).is_ok());
        assert!(require_args_range("test", &args, 2, 2).is_ok());
        assert!(require_args_range("test", &args, 3, 5).is_err());
    }

    #[test]
    fn test_require_string() {
        let s = Value::String(Arc::new("hello".to_string()));
        assert_eq!(require_string(&s).unwrap(), "hello");

        let i = Value::Int(42);
        assert!(require_string(&i).is_err());
    }

    #[test]
    fn test_require_int() {
        let i = Value::Int(42);
        assert_eq!(require_int(&i).unwrap(), 42);

        let s = Value::String(Arc::new("hello".to_string()));
        assert!(require_int(&s).is_err());
    }

    #[test]
    fn test_get_string_arg() {
        let args = vec![Value::String(Arc::new("hello".to_string()))];
        assert_eq!(get_string_arg("test", &args, 0).unwrap(), "hello");
        assert!(get_string_arg("test", &args, 1).is_err());
    }

    #[test]
    fn test_get_optional_string_arg() {
        let args = vec![Value::String(Arc::new("hello".to_string()))];
        assert_eq!(
            get_optional_string_arg(&args, 0).unwrap(),
            Some("hello".to_string())
        );
        assert_eq!(get_optional_string_arg(&args, 1).unwrap(), None);

        let args_with_none = vec![Value::None];
        assert_eq!(get_optional_string_arg(&args_with_none, 0).unwrap(), None);
    }
}
