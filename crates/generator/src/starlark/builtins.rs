use std::collections::{HashMap, HashSet};
use std::rc::Rc;
use std::cell::RefCell;
use super::value::{Value, BuiltinFn, HashableValue};
use super::generator::SchemaGenerator;
use blueprint_common::{SchemaOp, SchemaValue};

pub fn register_base_builtins(compiler: &mut SchemaGenerator) {
    compiler.register_builtin("len", builtin_len);
    compiler.register_builtin("str", builtin_str);
    compiler.register_builtin("repr", builtin_repr);
    compiler.register_builtin("int", builtin_int);
    compiler.register_builtin("float", builtin_float);
    compiler.register_builtin("bool", builtin_bool);
    compiler.register_builtin("bytes", builtin_bytes);
    compiler.register_builtin("list", builtin_list);
    compiler.register_builtin("dict", builtin_dict);
    compiler.register_builtin("set", builtin_set);
    compiler.register_builtin("tuple", builtin_tuple);
    compiler.register_builtin("range", builtin_range);
    compiler.register_builtin("enumerate", builtin_enumerate);
    compiler.register_builtin("zip", builtin_zip);
    compiler.register_builtin("sorted", builtin_sorted);
    compiler.register_builtin("reversed", builtin_reversed);
    compiler.register_builtin("min", builtin_min);
    compiler.register_builtin("max", builtin_max);
    compiler.register_builtin("sum", builtin_sum);
    compiler.register_builtin("abs", builtin_abs);
    compiler.register_builtin("all", builtin_all);
    compiler.register_builtin("any", builtin_any);
    compiler.register_builtin("hasattr", builtin_hasattr);
    compiler.register_builtin("getattr", builtin_getattr);
    compiler.register_builtin("type", builtin_type);
    compiler.register_builtin("print", builtin_print);
    compiler.register_builtin("fail", builtin_fail);
    compiler.register_builtin("hash", builtin_hash);
    compiler.register_builtin("dir", builtin_dir);
    compiler.register_builtin("filter", builtin_filter);
    compiler.register_builtin("map", builtin_map);
    compiler.register_builtin("struct", builtin_struct);
    compiler.register_builtin("partial", builtin_partial);

    let json_module = create_json_module();
    compiler.set_global("json", json_module);
}

fn make_builtin(f: fn(&mut SchemaGenerator, Vec<Value>, HashMap<String, Value>) -> Result<Value, String>) -> Value {
    Value::BuiltinFunction(Rc::new(f) as BuiltinFn)
}

fn create_json_module() -> Value {
    let mut json_dict = HashMap::new();
    json_dict.insert("encode".to_string(), make_builtin(builtin_json_encode));
    json_dict.insert("decode".to_string(), make_builtin(builtin_json_decode));
    Value::Dict(Rc::new(RefCell::new(json_dict)))
}

pub fn create_io_exports() -> HashMap<String, Value> {
    let mut exports = HashMap::new();
    exports.insert("read_file".to_string(), make_builtin(builtin_read_file));
    exports.insert("write_file".to_string(), make_builtin(builtin_write_file));
    exports.insert("append_file".to_string(), make_builtin(builtin_append_file));
    exports.insert("delete_file".to_string(), make_builtin(builtin_delete_file));
    exports.insert("file_exists".to_string(), make_builtin(builtin_file_exists));
    exports.insert("is_dir".to_string(), make_builtin(builtin_is_dir));
    exports.insert("is_file".to_string(), make_builtin(builtin_is_file));
    exports.insert("mkdir".to_string(), make_builtin(builtin_mkdir));
    exports.insert("rmdir".to_string(), make_builtin(builtin_rmdir));
    exports.insert("list_dir".to_string(), make_builtin(builtin_list_dir));
    exports.insert("copy_file".to_string(), make_builtin(builtin_copy_file));
    exports.insert("move_file".to_string(), make_builtin(builtin_move_file));
    exports.insert("file_size".to_string(), make_builtin(builtin_file_size));
    exports
}

pub fn create_http_exports() -> HashMap<String, Value> {
    let mut exports = HashMap::new();
    exports.insert("http_request".to_string(), make_builtin(builtin_http_request));
    exports
}

pub fn create_exec_exports() -> HashMap<String, Value> {
    let mut exports = HashMap::new();
    exports.insert("exec_run".to_string(), make_builtin(builtin_exec_run));
    exports.insert("exec_shell".to_string(), make_builtin(builtin_exec_shell));
    exports.insert("env_get".to_string(), make_builtin(builtin_env_get));
    exports
}

pub fn create_json_exports() -> HashMap<String, Value> {
    let mut exports = HashMap::new();
    exports.insert("json_encode".to_string(), make_builtin(builtin_json_encode));
    exports.insert("json_decode".to_string(), make_builtin(builtin_json_decode));
    exports
}

fn builtin_fail(_: &mut SchemaGenerator, args: Vec<Value>, _: HashMap<String, Value>) -> Result<Value, String> {
    let message = if args.is_empty() {
        "fail".to_string()
    } else {
        args.iter().map(|v| v.to_string_repr()).collect::<Vec<_>>().join(" ")
    };
    Err(message)
}

fn builtin_len(_: &mut SchemaGenerator, args: Vec<Value>, _: HashMap<String, Value>) -> Result<Value, String> {
    if args.len() != 1 {
        return Err(format!("len() takes exactly 1 argument ({} given)", args.len()));
    }
    match &args[0] {
        Value::String(s) => Ok(Value::Int(s.len() as i64)),
        Value::Bytes(b) => Ok(Value::Int(b.len() as i64)),
        Value::List(l) => Ok(Value::Int(l.borrow().len() as i64)),
        Value::Dict(d) => Ok(Value::Int(d.borrow().len() as i64)),
        Value::Set(s) => Ok(Value::Int(s.borrow().len() as i64)),
        Value::Tuple(t) => Ok(Value::Int(t.len() as i64)),
        v => Err(format!("object of type '{}' has no len()", v.type_name())),
    }
}

fn builtin_str(_: &mut SchemaGenerator, args: Vec<Value>, _: HashMap<String, Value>) -> Result<Value, String> {
    if args.len() != 1 {
        return Err(format!("str() takes exactly 1 argument ({} given)", args.len()));
    }
    Ok(Value::String(args[0].to_string_repr()))
}

fn builtin_repr(_: &mut SchemaGenerator, args: Vec<Value>, _: HashMap<String, Value>) -> Result<Value, String> {
    if args.len() != 1 {
        return Err(format!("repr() takes exactly 1 argument ({} given)", args.len()));
    }
    Ok(Value::String(args[0].to_repr()))
}

fn builtin_int(_: &mut SchemaGenerator, args: Vec<Value>, _: HashMap<String, Value>) -> Result<Value, String> {
    if args.is_empty() || args.len() > 2 {
        return Err(format!("int() takes 1 or 2 arguments ({} given)", args.len()));
    }
    match &args[0] {
        Value::Int(n) => Ok(Value::Int(*n)),
        Value::Float(f) => Ok(Value::Int(*f as i64)),
        Value::Bool(b) => Ok(Value::Int(if *b { 1 } else { 0 })),
        Value::String(s) => {
            let base = if args.len() == 2 {
                match &args[1] {
                    Value::Int(b) => *b as u32,
                    _ => return Err("int() base must be an integer".to_string()),
                }
            } else {
                10
            };
            let s_trimmed = s.trim();
            if base == 0 {
                if s_trimmed.starts_with("0x") || s_trimmed.starts_with("0X") {
                    i64::from_str_radix(&s_trimmed[2..], 16)
                        .map(Value::Int)
                        .map_err(|_| format!("invalid literal for int() with base 0: '{}'", s))
                } else if s_trimmed.starts_with("0o") || s_trimmed.starts_with("0O") {
                    i64::from_str_radix(&s_trimmed[2..], 8)
                        .map(Value::Int)
                        .map_err(|_| format!("invalid literal for int() with base 0: '{}'", s))
                } else if s_trimmed.starts_with("0b") || s_trimmed.starts_with("0B") {
                    i64::from_str_radix(&s_trimmed[2..], 2)
                        .map(Value::Int)
                        .map_err(|_| format!("invalid literal for int() with base 0: '{}'", s))
                } else {
                    i64::from_str_radix(s_trimmed, 10)
                        .map(Value::Int)
                        .map_err(|_| format!("invalid literal for int() with base 0: '{}'", s))
                }
            } else {
                i64::from_str_radix(s_trimmed, base)
                    .map(Value::Int)
                    .map_err(|_| format!("invalid literal for int() with base {}: '{}'", base, s))
            }
        }
        v => Err(format!("int() argument must be a string or number, not '{}'", v.type_name())),
    }
}

fn builtin_float(_: &mut SchemaGenerator, args: Vec<Value>, _: HashMap<String, Value>) -> Result<Value, String> {
    if args.len() != 1 {
        return Err(format!("float() takes exactly 1 argument ({} given)", args.len()));
    }
    match &args[0] {
        Value::Int(n) => Ok(Value::Float(*n as f64)),
        Value::Float(f) => Ok(Value::Float(*f)),
        Value::Bool(b) => Ok(Value::Float(if *b { 1.0 } else { 0.0 })),
        Value::String(s) => s.trim().parse::<f64>()
            .map(Value::Float)
            .map_err(|_| format!("could not convert string to float: '{}'", s)),
        v => Err(format!("float() argument must be a string or number, not '{}'", v.type_name())),
    }
}

fn builtin_bool(_: &mut SchemaGenerator, args: Vec<Value>, _: HashMap<String, Value>) -> Result<Value, String> {
    if args.len() > 1 {
        return Err(format!("bool() takes at most 1 argument ({} given)", args.len()));
    }
    if args.is_empty() {
        Ok(Value::Bool(false))
    } else {
        Ok(Value::Bool(args[0].is_truthy()))
    }
}

fn builtin_bytes(_: &mut SchemaGenerator, args: Vec<Value>, _: HashMap<String, Value>) -> Result<Value, String> {
    if args.len() > 1 {
        return Err(format!("bytes() takes at most 1 argument ({} given)", args.len()));
    }
    if args.is_empty() {
        return Ok(Value::Bytes(Vec::new()));
    }
    match &args[0] {
        Value::Bytes(b) => Ok(Value::Bytes(b.clone())),
        Value::String(s) => Ok(Value::Bytes(s.as_bytes().to_vec())),
        Value::List(l) => {
            let mut result = Vec::new();
            for v in l.borrow().iter() {
                match v {
                    Value::Int(n) => {
                        if *n < 0 || *n > 255 {
                            return Err(format!("bytes must be in range(0, 256), got {}", n));
                        }
                        result.push(*n as u8);
                    }
                    _ => return Err(format!("'{}' object cannot be interpreted as an integer", v.type_name())),
                }
            }
            Ok(Value::Bytes(result))
        }
        Value::Tuple(t) => {
            let mut result = Vec::new();
            for v in t.iter() {
                match v {
                    Value::Int(n) => {
                        if *n < 0 || *n > 255 {
                            return Err(format!("bytes must be in range(0, 256), got {}", n));
                        }
                        result.push(*n as u8);
                    }
                    _ => return Err(format!("'{}' object cannot be interpreted as an integer", v.type_name())),
                }
            }
            Ok(Value::Bytes(result))
        }
        v => Err(format!("cannot convert '{}' to bytes", v.type_name())),
    }
}

fn builtin_list(_: &mut SchemaGenerator, args: Vec<Value>, _: HashMap<String, Value>) -> Result<Value, String> {
    if args.len() > 1 {
        return Err(format!("list() takes at most 1 argument ({} given)", args.len()));
    }
    if args.is_empty() {
        return Ok(Value::List(Rc::new(RefCell::new(Vec::new()))));
    }
    match &args[0] {
        Value::List(l) => Ok(Value::List(Rc::new(RefCell::new(l.borrow().clone())))),
        Value::Tuple(t) => Ok(Value::List(Rc::new(RefCell::new(t.clone())))),
        Value::String(s) => {
            let chars: Vec<Value> = s.chars().map(|c| Value::String(c.to_string())).collect();
            Ok(Value::List(Rc::new(RefCell::new(chars))))
        }
        v => Err(format!("'{}' object is not iterable", v.type_name())),
    }
}

fn builtin_dict(_: &mut SchemaGenerator, args: Vec<Value>, kwargs: HashMap<String, Value>) -> Result<Value, String> {
    let mut result = HashMap::new();

    if !args.is_empty() {
        let pairs = match &args[0] {
            Value::List(l) => l.borrow().clone(),
            Value::Tuple(t) => t.clone(),
            Value::Dict(d) => {
                for (k, v) in d.borrow().iter() {
                    result.insert(k.clone(), v.clone());
                }
                Vec::new()
            }
            _ => return Err("dict() argument must be iterable of key-value pairs".to_string()),
        };
        for item in pairs {
            match item {
                Value::Tuple(t) if t.len() == 2 => {
                    let key = match &t[0] {
                        Value::String(s) => s.clone(),
                        _ => return Err("dict keys must be strings".to_string()),
                    };
                    result.insert(key, t[1].clone());
                }
                Value::List(l) if l.borrow().len() == 2 => {
                    let borrowed = l.borrow();
                    let key = match &borrowed[0] {
                        Value::String(s) => s.clone(),
                        _ => return Err("dict keys must be strings".to_string()),
                    };
                    result.insert(key, borrowed[1].clone());
                }
                _ => return Err("dict() items must be key-value pairs".to_string()),
            }
        }
    }

    for (k, v) in kwargs {
        result.insert(k, v);
    }

    Ok(Value::Dict(Rc::new(RefCell::new(result))))
}

fn builtin_set(_: &mut SchemaGenerator, args: Vec<Value>, _: HashMap<String, Value>) -> Result<Value, String> {
    if args.len() > 1 {
        return Err(format!("set() takes at most 1 argument ({} given)", args.len()));
    }
    if args.is_empty() {
        return Ok(Value::Set(Rc::new(RefCell::new(HashSet::new()))));
    }
    let mut result = HashSet::new();
    match &args[0] {
        Value::List(l) => {
            for v in l.borrow().iter() {
                let h = HashableValue::from_value(v)?;
                result.insert(h);
            }
        }
        Value::Tuple(t) => {
            for v in t.iter() {
                let h = HashableValue::from_value(v)?;
                result.insert(h);
            }
        }
        Value::Set(s) => {
            result = s.borrow().clone();
        }
        Value::String(s) => {
            for c in s.chars() {
                result.insert(HashableValue::String(c.to_string()));
            }
        }
        v => return Err(format!("'{}' object is not iterable", v.type_name())),
    }
    Ok(Value::Set(Rc::new(RefCell::new(result))))
}

fn builtin_tuple(_: &mut SchemaGenerator, args: Vec<Value>, _: HashMap<String, Value>) -> Result<Value, String> {
    if args.len() > 1 {
        return Err(format!("tuple() takes at most 1 argument ({} given)", args.len()));
    }
    if args.is_empty() {
        return Ok(Value::Tuple(Vec::new()));
    }
    match &args[0] {
        Value::List(l) => Ok(Value::Tuple(l.borrow().clone())),
        Value::Tuple(t) => Ok(Value::Tuple(t.clone())),
        Value::String(s) => {
            let chars: Vec<Value> = s.chars().map(|c| Value::String(c.to_string())).collect();
            Ok(Value::Tuple(chars))
        }
        v => Err(format!("'{}' object is not iterable", v.type_name())),
    }
}

fn builtin_range(_: &mut SchemaGenerator, args: Vec<Value>, _: HashMap<String, Value>) -> Result<Value, String> {
    let (start, stop, step) = match args.len() {
        1 => (0, extract_int(&args[0])?, 1),
        2 => (extract_int(&args[0])?, extract_int(&args[1])?, 1),
        3 => (extract_int(&args[0])?, extract_int(&args[1])?, extract_int(&args[2])?),
        n => return Err(format!("range() takes 1 to 3 arguments ({} given)", n)),
    };

    if step == 0 {
        return Err("range() step argument must not be zero".to_string());
    }

    let mut result = Vec::new();
    let mut i = start;
    if step > 0 {
        while i < stop {
            result.push(Value::Int(i));
            i += step;
        }
    } else {
        while i > stop {
            result.push(Value::Int(i));
            i += step;
        }
    }

    Ok(Value::List(Rc::new(RefCell::new(result))))
}

fn builtin_enumerate(_: &mut SchemaGenerator, args: Vec<Value>, _: HashMap<String, Value>) -> Result<Value, String> {
    if args.is_empty() || args.len() > 2 {
        return Err(format!("enumerate() takes 1 or 2 arguments ({} given)", args.len()));
    }

    let start = if args.len() == 2 {
        extract_int(&args[1])?
    } else {
        0
    };

    let items = extract_iterable(&args[0])?;
    let result: Vec<Value> = items.into_iter()
        .enumerate()
        .map(|(i, v)| Value::Tuple(vec![Value::Int(start + i as i64), v]))
        .collect();

    Ok(Value::List(Rc::new(RefCell::new(result))))
}

fn builtin_zip(_: &mut SchemaGenerator, args: Vec<Value>, _: HashMap<String, Value>) -> Result<Value, String> {
    if args.is_empty() {
        return Ok(Value::List(Rc::new(RefCell::new(Vec::new()))));
    }

    let iterables: Result<Vec<Vec<Value>>, String> = args.iter()
        .map(extract_iterable)
        .collect();
    let iterables = iterables?;

    let min_len = iterables.iter().map(|v| v.len()).min().unwrap_or(0);
    let mut result = Vec::new();

    for i in 0..min_len {
        let tuple: Vec<Value> = iterables.iter().map(|v| v[i].clone()).collect();
        result.push(Value::Tuple(tuple));
    }

    Ok(Value::List(Rc::new(RefCell::new(result))))
}

fn builtin_sorted(_: &mut SchemaGenerator, args: Vec<Value>, kwargs: HashMap<String, Value>) -> Result<Value, String> {
    if args.len() != 1 {
        return Err(format!("sorted() takes exactly 1 positional argument ({} given)", args.len()));
    }

    let reverse = kwargs.get("reverse")
        .map(|v| v.is_truthy())
        .unwrap_or(false);

    let mut items = extract_iterable(&args[0])?;
    items.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));

    if reverse {
        items.reverse();
    }

    Ok(Value::List(Rc::new(RefCell::new(items))))
}

fn builtin_reversed(_: &mut SchemaGenerator, args: Vec<Value>, _: HashMap<String, Value>) -> Result<Value, String> {
    if args.len() != 1 {
        return Err(format!("reversed() takes exactly 1 argument ({} given)", args.len()));
    }

    let mut items = extract_iterable(&args[0])?;
    items.reverse();
    Ok(Value::List(Rc::new(RefCell::new(items))))
}

fn builtin_min(compiler: &mut SchemaGenerator, args: Vec<Value>, kwargs: HashMap<String, Value>) -> Result<Value, String> {
    if args.is_empty() {
        return Err("min() requires at least 1 argument".to_string());
    }

    let items = if args.len() == 1 {
        extract_iterable(&args[0])?
    } else {
        args
    };

    if items.is_empty() {
        return Err("min() arg is an empty sequence".to_string());
    }

    if let Some(key_fn) = kwargs.get("key") {
        let mut min_item: Option<Value> = None;
        let mut min_key: Option<Value> = None;
        for item in items {
            let key_value = compiler.call_value(&key_fn.clone(), vec![item.clone()], HashMap::new())?;
            if min_key.is_none() || key_value.partial_cmp(min_key.as_ref().unwrap()).map(|o| o == std::cmp::Ordering::Less).unwrap_or(false) {
                min_key = Some(key_value);
                min_item = Some(item);
            }
        }
        min_item.ok_or_else(|| "min() arg is an empty sequence".to_string())
    } else {
        items.into_iter()
            .min_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal))
            .ok_or_else(|| "min() arg is an empty sequence".to_string())
    }
}

fn builtin_max(_: &mut SchemaGenerator, args: Vec<Value>, _: HashMap<String, Value>) -> Result<Value, String> {
    if args.is_empty() {
        return Err("max() requires at least 1 argument".to_string());
    }

    let items = if args.len() == 1 {
        extract_iterable(&args[0])?
    } else {
        args
    };

    if items.is_empty() {
        return Err("max() arg is an empty sequence".to_string());
    }

    items.into_iter()
        .max_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal))
        .ok_or_else(|| "max() arg is an empty sequence".to_string())
}

fn builtin_sum(_: &mut SchemaGenerator, args: Vec<Value>, _: HashMap<String, Value>) -> Result<Value, String> {
    if args.is_empty() || args.len() > 2 {
        return Err(format!("sum() takes 1 or 2 arguments ({} given)", args.len()));
    }

    let start = if args.len() == 2 {
        match &args[1] {
            Value::Int(n) => Value::Int(*n),
            Value::Float(f) => Value::Float(*f),
            _ => return Err("sum() start must be a number".to_string()),
        }
    } else {
        Value::Int(0)
    };

    let items = extract_iterable(&args[0])?;
    let mut result = start;

    for item in items {
        result = add_values(&result, &item)?;
    }

    Ok(result)
}

fn builtin_abs(_: &mut SchemaGenerator, args: Vec<Value>, _: HashMap<String, Value>) -> Result<Value, String> {
    if args.len() != 1 {
        return Err(format!("abs() takes exactly 1 argument ({} given)", args.len()));
    }

    match &args[0] {
        Value::Int(n) => Ok(Value::Int(n.abs())),
        Value::Float(f) => Ok(Value::Float(f.abs())),
        v => Err(format!("bad operand type for abs(): '{}'", v.type_name())),
    }
}

fn builtin_all(_: &mut SchemaGenerator, args: Vec<Value>, _: HashMap<String, Value>) -> Result<Value, String> {
    if args.len() != 1 {
        return Err(format!("all() takes exactly 1 argument ({} given)", args.len()));
    }

    let items = extract_iterable(&args[0])?;
    Ok(Value::Bool(items.iter().all(|v| v.is_truthy())))
}

fn builtin_any(_: &mut SchemaGenerator, args: Vec<Value>, _: HashMap<String, Value>) -> Result<Value, String> {
    if args.len() != 1 {
        return Err(format!("any() takes exactly 1 argument ({} given)", args.len()));
    }

    let items = extract_iterable(&args[0])?;
    Ok(Value::Bool(items.iter().any(|v| v.is_truthy())))
}

fn get_type_methods(type_name: &str) -> Vec<&'static str> {
    match type_name {
        "string" => vec![
            "upper", "lower", "strip", "lstrip", "rstrip", "capitalize", "title", "swapcase",
            "isalpha", "isdigit", "isalnum", "isspace", "isupper", "islower", "istitle",
            "split", "rsplit", "splitlines", "join", "replace", "find", "rfind", "index", "rindex",
            "count", "startswith", "endswith", "format", "removeprefix", "removesuffix", "elems",
            "partition", "rpartition"
        ],
        "list" => vec![
            "append", "extend", "insert", "pop", "remove", "clear", "index"
        ],
        "dict" => vec![
            "keys", "values", "items", "get", "pop", "clear", "update", "setdefault", "popitem"
        ],
        "set" => vec!["add", "remove", "discard", "clear", "union", "intersection", "difference"],
        "bytes" => vec!["elems"],
        _ => vec![],
    }
}

fn builtin_hasattr(_: &mut SchemaGenerator, args: Vec<Value>, _: HashMap<String, Value>) -> Result<Value, String> {
    if args.len() != 2 {
        return Err(format!("hasattr() takes exactly 2 arguments ({} given)", args.len()));
    }
    let attr_name = match &args[1] {
        Value::String(s) => s.as_str(),
        _ => return Err("hasattr() attribute name must be a string".to_string()),
    };
    let type_name = args[0].type_name();
    let methods = get_type_methods(type_name);
    Ok(Value::Bool(methods.contains(&attr_name)))
}

fn builtin_getattr(compiler: &mut SchemaGenerator, args: Vec<Value>, _: HashMap<String, Value>) -> Result<Value, String> {
    if args.len() < 2 || args.len() > 3 {
        return Err(format!("getattr() takes 2 or 3 arguments ({} given)", args.len()));
    }
    let attr_name = match &args[1] {
        Value::String(s) => s.clone(),
        _ => return Err("getattr() attribute name must be a string".to_string()),
    };
    let type_name = args[0].type_name();
    let methods = get_type_methods(type_name);
    if methods.contains(&attr_name.as_str()) {
        match &args[0] {
            Value::String(s) => compiler.string_method_value(s, &attr_name),
            Value::List(l) => compiler.list_method_value(l, &attr_name),
            Value::Dict(d) => compiler.dict_method_value(d, &attr_name),
            _ => {
                if args.len() == 3 {
                    Ok(args[2].clone())
                } else {
                    Err(format!("'{}' object has no attribute '{}'", type_name, attr_name))
                }
            }
        }
    } else if args.len() == 3 {
        Ok(args[2].clone())
    } else {
        Err(format!("'{}' object has no attribute '{}'", type_name, attr_name))
    }
}

fn builtin_type(_: &mut SchemaGenerator, args: Vec<Value>, _: HashMap<String, Value>) -> Result<Value, String> {
    if args.len() != 1 {
        return Err(format!("type() takes exactly 1 argument ({} given)", args.len()));
    }
    Ok(Value::String(args[0].type_name().to_string()))
}

fn builtin_struct(_: &mut SchemaGenerator, args: Vec<Value>, kwargs: HashMap<String, Value>) -> Result<Value, String> {
    if !args.is_empty() {
        return Err("struct() takes no positional arguments".to_string());
    }
    Ok(Value::Struct(kwargs))
}

fn builtin_partial(_: &mut SchemaGenerator, args: Vec<Value>, kwargs: HashMap<String, Value>) -> Result<Value, String> {
    if args.is_empty() {
        return Err("partial() requires at least 1 argument".to_string());
    }
    let func = match &args[0] {
        Value::Function(f) => f.clone(),
        _ => return Err("partial() first argument must be a function".to_string()),
    };
    let bound_args = args[1..].to_vec();
    Ok(Value::Partial {
        func,
        bound_args,
        bound_kwargs: kwargs,
    })
}

fn builtin_print(compiler: &mut SchemaGenerator, args: Vec<Value>, _: HashMap<String, Value>) -> Result<Value, String> {
    let has_dynamic = args.iter().any(|v| v.is_dynamic());

    if has_dynamic {
        let message = if args.len() == 1 {
            args[0].to_schema_value()
        } else {
            let mut combined = args[0].to_schema_value();
            for arg in args.iter().skip(1) {
                let space = SchemaValue::literal_string(" ");
                let space_id = compiler.add_schema_op(SchemaOp::Concat { left: combined, right: space });
                combined = SchemaValue::OpRef { id: space_id, path: Vec::new() };
                let concat_id = compiler.add_schema_op(SchemaOp::Concat { left: combined, right: arg.to_schema_value() });
                combined = SchemaValue::OpRef { id: concat_id, path: Vec::new() };
            }
            combined
        };
        let op = SchemaOp::BpPrint { message };
        let id = compiler.add_schema_op(op);
        Ok(Value::OpRef(id))
    } else {
        let output: Vec<String> = args.iter().map(|v| v.to_string_repr()).collect();
        println!("{}", output.join(" "));
        Ok(Value::None)
    }
}

fn builtin_hash(_: &mut SchemaGenerator, args: Vec<Value>, _: HashMap<String, Value>) -> Result<Value, String> {
    if args.len() != 1 {
        return Err(format!("hash() takes exactly 1 argument ({} given)", args.len()));
    }
    use std::hash::{Hash, Hasher};
    use std::collections::hash_map::DefaultHasher;
    let hashable = HashableValue::from_value(&args[0])?;
    let mut hasher = DefaultHasher::new();
    hashable.hash(&mut hasher);
    Ok(Value::Int(hasher.finish() as i64))
}

fn builtin_dir(_: &mut SchemaGenerator, args: Vec<Value>, _: HashMap<String, Value>) -> Result<Value, String> {
    if args.len() != 1 {
        return Err(format!("dir() takes exactly 1 argument ({} given)", args.len()));
    }
    let type_name = args[0].type_name();
    let methods: Vec<Value> = get_type_methods(type_name)
        .iter()
        .map(|s| Value::String(s.to_string()))
        .collect();
    Ok(Value::List(Rc::new(RefCell::new(methods))))
}

fn builtin_read_file(compiler: &mut SchemaGenerator, args: Vec<Value>, _: HashMap<String, Value>) -> Result<Value, String> {
    if args.len() != 1 {
        return Err(format!("read_file() takes exactly 1 argument ({} given)", args.len()));
    }
    let path = args[0].to_schema_value();
    let op = SchemaOp::IoReadFile { path };
    let id = compiler.add_schema_op(op);
    Ok(Value::OpRef(id))
}

fn builtin_write_file(compiler: &mut SchemaGenerator, args: Vec<Value>, _: HashMap<String, Value>) -> Result<Value, String> {
    if args.len() != 2 {
        return Err(format!("write_file() takes exactly 2 arguments ({} given)", args.len()));
    }
    let path = args[0].to_schema_value();
    let content = args[1].to_schema_value();
    let op = SchemaOp::IoWriteFile { path, content };
    let id = compiler.add_schema_op(op);
    Ok(Value::OpRef(id))
}

fn builtin_append_file(compiler: &mut SchemaGenerator, args: Vec<Value>, _: HashMap<String, Value>) -> Result<Value, String> {
    if args.len() != 2 {
        return Err(format!("append_file() takes exactly 2 arguments ({} given)", args.len()));
    }
    let path = args[0].to_schema_value();
    let content = args[1].to_schema_value();
    let op = SchemaOp::IoAppendFile { path, content };
    let id = compiler.add_schema_op(op);
    Ok(Value::OpRef(id))
}

fn builtin_delete_file(compiler: &mut SchemaGenerator, args: Vec<Value>, _: HashMap<String, Value>) -> Result<Value, String> {
    if args.len() != 1 {
        return Err(format!("delete_file() takes exactly 1 argument ({} given)", args.len()));
    }
    let path = args[0].to_schema_value();
    let op = SchemaOp::IoDeleteFile { path };
    let id = compiler.add_schema_op(op);
    Ok(Value::OpRef(id))
}

fn builtin_file_exists(compiler: &mut SchemaGenerator, args: Vec<Value>, _: HashMap<String, Value>) -> Result<Value, String> {
    if args.len() != 1 {
        return Err(format!("file_exists() takes exactly 1 argument ({} given)", args.len()));
    }
    let path = args[0].to_schema_value();
    let op = SchemaOp::IoFileExists { path };
    let id = compiler.add_schema_op(op);
    Ok(Value::OpRef(id))
}

fn builtin_is_dir(compiler: &mut SchemaGenerator, args: Vec<Value>, _: HashMap<String, Value>) -> Result<Value, String> {
    if args.len() != 1 {
        return Err(format!("is_dir() takes exactly 1 argument ({} given)", args.len()));
    }
    let path = args[0].to_schema_value();
    let op = SchemaOp::IoIsDir { path };
    let id = compiler.add_schema_op(op);
    Ok(Value::OpRef(id))
}

fn builtin_is_file(compiler: &mut SchemaGenerator, args: Vec<Value>, _: HashMap<String, Value>) -> Result<Value, String> {
    if args.len() != 1 {
        return Err(format!("is_file() takes exactly 1 argument ({} given)", args.len()));
    }
    let path = args[0].to_schema_value();
    let op = SchemaOp::IoIsFile { path };
    let id = compiler.add_schema_op(op);
    Ok(Value::OpRef(id))
}

fn builtin_mkdir(compiler: &mut SchemaGenerator, args: Vec<Value>, kwargs: HashMap<String, Value>) -> Result<Value, String> {
    if args.len() != 1 {
        return Err(format!("mkdir() takes exactly 1 argument ({} given)", args.len()));
    }
    let path = args[0].to_schema_value();
    let recursive = kwargs.get("recursive").map(|v| v.is_truthy()).unwrap_or(false);
    let op = SchemaOp::IoMkdir { path, recursive };
    let id = compiler.add_schema_op(op);
    Ok(Value::OpRef(id))
}

fn builtin_rmdir(compiler: &mut SchemaGenerator, args: Vec<Value>, kwargs: HashMap<String, Value>) -> Result<Value, String> {
    if args.len() != 1 {
        return Err(format!("rmdir() takes exactly 1 argument ({} given)", args.len()));
    }
    let path = args[0].to_schema_value();
    let recursive = kwargs.get("recursive").map(|v| v.is_truthy()).unwrap_or(false);
    let op = SchemaOp::IoRmdir { path, recursive };
    let id = compiler.add_schema_op(op);
    Ok(Value::OpRef(id))
}

fn builtin_list_dir(compiler: &mut SchemaGenerator, args: Vec<Value>, _: HashMap<String, Value>) -> Result<Value, String> {
    if args.len() != 1 {
        return Err(format!("list_dir() takes exactly 1 argument ({} given)", args.len()));
    }
    let path = args[0].to_schema_value();
    let op = SchemaOp::IoListDir { path };
    let id = compiler.add_schema_op(op);
    Ok(Value::OpRef(id))
}

fn builtin_copy_file(compiler: &mut SchemaGenerator, args: Vec<Value>, _: HashMap<String, Value>) -> Result<Value, String> {
    if args.len() != 2 {
        return Err(format!("copy_file() takes exactly 2 arguments ({} given)", args.len()));
    }
    let src = args[0].to_schema_value();
    let dst = args[1].to_schema_value();
    let op = SchemaOp::IoCopyFile { src, dst };
    let id = compiler.add_schema_op(op);
    Ok(Value::OpRef(id))
}

fn builtin_move_file(compiler: &mut SchemaGenerator, args: Vec<Value>, _: HashMap<String, Value>) -> Result<Value, String> {
    if args.len() != 2 {
        return Err(format!("move_file() takes exactly 2 arguments ({} given)", args.len()));
    }
    let src = args[0].to_schema_value();
    let dst = args[1].to_schema_value();
    let op = SchemaOp::IoMoveFile { src, dst };
    let id = compiler.add_schema_op(op);
    Ok(Value::OpRef(id))
}

fn builtin_file_size(compiler: &mut SchemaGenerator, args: Vec<Value>, _: HashMap<String, Value>) -> Result<Value, String> {
    if args.len() != 1 {
        return Err(format!("file_size() takes exactly 1 argument ({} given)", args.len()));
    }
    let path = args[0].to_schema_value();
    let op = SchemaOp::IoFileSize { path };
    let id = compiler.add_schema_op(op);
    Ok(Value::OpRef(id))
}

fn builtin_http_request(compiler: &mut SchemaGenerator, args: Vec<Value>, kwargs: HashMap<String, Value>) -> Result<Value, String> {
    if args.len() != 2 {
        return Err(format!("http_request() takes exactly 2 arguments ({} given)", args.len()));
    }
    let method = args[0].to_schema_value();
    let url = args[1].to_schema_value();
    let body = kwargs.get("body").map(|v| v.to_schema_value())
        .unwrap_or_else(|| blueprint_common::SchemaValue::Literal(blueprint_common::RecordedValue::None));
    let headers = kwargs.get("headers").map(|v| v.to_schema_value())
        .unwrap_or_else(|| blueprint_common::SchemaValue::Literal(blueprint_common::RecordedValue::None));
    let op = SchemaOp::HttpRequest { method, url, body, headers };
    let id = compiler.add_schema_op(op);
    Ok(Value::OpRef(id))
}

fn builtin_exec_run(compiler: &mut SchemaGenerator, args: Vec<Value>, _: HashMap<String, Value>) -> Result<Value, String> {
    if args.is_empty() || args.len() > 2 {
        return Err(format!("exec_run() takes 1 or 2 arguments ({} given)", args.len()));
    }
    let command = args[0].to_schema_value();
    let exec_args = if args.len() > 1 {
        args[1].to_schema_value()
    } else {
        blueprint_common::SchemaValue::Literal(blueprint_common::RecordedValue::None)
    };
    let op = SchemaOp::ExecRun { command, args: exec_args };
    let id = compiler.add_schema_op(op);
    Ok(Value::OpRef(id))
}

fn builtin_exec_shell(compiler: &mut SchemaGenerator, args: Vec<Value>, _: HashMap<String, Value>) -> Result<Value, String> {
    if args.len() != 1 {
        return Err(format!("exec_shell() takes exactly 1 argument ({} given)", args.len()));
    }
    let command = args[0].to_schema_value();
    let op = SchemaOp::ExecShell { command };
    let id = compiler.add_schema_op(op);
    Ok(Value::OpRef(id))
}

fn builtin_env_get(compiler: &mut SchemaGenerator, args: Vec<Value>, _: HashMap<String, Value>) -> Result<Value, String> {
    if args.is_empty() || args.len() > 2 {
        return Err(format!("env_get() takes 1 or 2 arguments ({} given)", args.len()));
    }
    let name = args[0].to_schema_value();
    let default = if args.len() > 1 {
        args[1].to_schema_value()
    } else {
        blueprint_common::SchemaValue::Literal(blueprint_common::RecordedValue::None)
    };
    let op = SchemaOp::ExecEnv { name, default };
    let id = compiler.add_schema_op(op);
    Ok(Value::OpRef(id))
}

fn builtin_json_encode(compiler: &mut SchemaGenerator, args: Vec<Value>, _: HashMap<String, Value>) -> Result<Value, String> {
    if args.len() != 1 {
        return Err(format!("json_encode() takes exactly 1 argument ({} given)", args.len()));
    }
    if !args[0].contains_dynamic() {
        let json_str = value_to_json(&args[0])?;
        return Ok(Value::String(json_str));
    }
    let value = args[0].to_schema_value();
    let op = SchemaOp::JsonEncode { value };
    let id = compiler.add_schema_op(op);
    Ok(Value::OpRef(id))
}

fn builtin_json_decode(compiler: &mut SchemaGenerator, args: Vec<Value>, _: HashMap<String, Value>) -> Result<Value, String> {
    if args.len() != 1 {
        return Err(format!("json_decode() takes exactly 1 argument ({} given)", args.len()));
    }
    if let Value::String(s) = &args[0] {
        let value = json_to_value(s)?;
        return Ok(value);
    }
    let string = args[0].to_schema_value();
    let op = SchemaOp::JsonDecode { string };
    let id = compiler.add_schema_op(op);
    Ok(Value::OpRef(id))
}

fn value_to_json(v: &Value) -> Result<String, String> {
    match v {
        Value::None => Ok("null".to_string()),
        Value::Bool(b) => Ok(if *b { "true".to_string() } else { "false".to_string() }),
        Value::Int(n) => Ok(n.to_string()),
        Value::Float(f) => Ok(f.to_string()),
        Value::String(s) => {
            let escaped = s.replace('\\', "\\\\").replace('"', "\\\"").replace('\n', "\\n").replace('\r', "\\r").replace('\t', "\\t");
            Ok(format!("\"{}\"", escaped))
        }
        Value::List(l) => {
            let items: Result<Vec<String>, String> = l.borrow().iter().map(value_to_json).collect();
            Ok(format!("[{}]", items?.join(",")))
        }
        Value::Dict(d) => {
            let items: Result<Vec<String>, String> = d.borrow().iter()
                .map(|(k, v)| {
                    let key_json = format!("\"{}\"", k.replace('\\', "\\\\").replace('"', "\\\""));
                    let val_json = value_to_json(v)?;
                    Ok(format!("{}:{}", key_json, val_json))
                })
                .collect();
            Ok(format!("{{{}}}", items?.join(",")))
        }
        Value::Tuple(t) => {
            let items: Result<Vec<String>, String> = t.iter().map(value_to_json).collect();
            Ok(format!("[{}]", items?.join(",")))
        }
        _ => Err(format!("Object of type '{}' is not JSON serializable", v.type_name())),
    }
}

fn json_to_value(s: &str) -> Result<Value, String> {
    let trimmed = s.trim();
    if trimmed == "null" {
        return Ok(Value::None);
    }
    if trimmed == "true" {
        return Ok(Value::Bool(true));
    }
    if trimmed == "false" {
        return Ok(Value::Bool(false));
    }
    if trimmed.starts_with('"') && trimmed.ends_with('"') {
        let inner = &trimmed[1..trimmed.len()-1];
        let unescaped = inner.replace("\\\"", "\"").replace("\\\\", "\\").replace("\\n", "\n").replace("\\r", "\r").replace("\\t", "\t");
        return Ok(Value::String(unescaped));
    }
    if let Ok(n) = trimmed.parse::<i64>() {
        return Ok(Value::Int(n));
    }
    if let Ok(f) = trimmed.parse::<f64>() {
        return Ok(Value::Float(f));
    }
    if trimmed.starts_with('[') && trimmed.ends_with(']') {
        let inner = &trimmed[1..trimmed.len()-1].trim();
        if inner.is_empty() {
            return Ok(Value::List(Rc::new(RefCell::new(Vec::new()))));
        }
        let items = parse_json_array(inner)?;
        let values: Result<Vec<Value>, String> = items.iter().map(|s| json_to_value(s)).collect();
        return Ok(Value::List(Rc::new(RefCell::new(values?))));
    }
    if trimmed.starts_with('{') && trimmed.ends_with('}') {
        let inner = &trimmed[1..trimmed.len()-1].trim();
        if inner.is_empty() {
            return Ok(Value::Dict(Rc::new(RefCell::new(HashMap::new()))));
        }
        let pairs = parse_json_object(inner)?;
        let mut dict = HashMap::new();
        for (k, v) in pairs {
            dict.insert(k, json_to_value(&v)?);
        }
        return Ok(Value::Dict(Rc::new(RefCell::new(dict))));
    }
    Err(format!("Invalid JSON: {}", s))
}

fn parse_json_array(s: &str) -> Result<Vec<String>, String> {
    let mut items = Vec::new();
    let mut depth = 0;
    let mut in_string = false;
    let mut escape = false;
    let mut start = 0;

    for (i, c) in s.char_indices() {
        if escape {
            escape = false;
            continue;
        }
        if c == '\\' && in_string {
            escape = true;
            continue;
        }
        if c == '"' {
            in_string = !in_string;
            continue;
        }
        if in_string {
            continue;
        }
        match c {
            '[' | '{' => depth += 1,
            ']' | '}' => depth -= 1,
            ',' if depth == 0 => {
                items.push(s[start..i].trim().to_string());
                start = i + 1;
            }
            _ => {}
        }
    }
    if start < s.len() {
        items.push(s[start..].trim().to_string());
    }
    Ok(items)
}

fn parse_json_object(s: &str) -> Result<Vec<(String, String)>, String> {
    let mut pairs = Vec::new();
    let mut depth = 0;
    let mut in_string = false;
    let mut escape = false;
    let mut start = 0;

    for (i, c) in s.char_indices() {
        if escape {
            escape = false;
            continue;
        }
        if c == '\\' && in_string {
            escape = true;
            continue;
        }
        if c == '"' {
            in_string = !in_string;
            continue;
        }
        if in_string {
            continue;
        }
        match c {
            '[' | '{' => depth += 1,
            ']' | '}' => depth -= 1,
            ',' if depth == 0 => {
                let pair = s[start..i].trim();
                let (key, val) = parse_key_value(pair)?;
                pairs.push((key, val));
                start = i + 1;
            }
            _ => {}
        }
    }
    if start < s.len() {
        let pair = s[start..].trim();
        let (key, val) = parse_key_value(pair)?;
        pairs.push((key, val));
    }
    Ok(pairs)
}

fn parse_key_value(s: &str) -> Result<(String, String), String> {
    let mut in_string = false;
    let mut escape = false;
    let mut colon_pos = None;

    for (i, c) in s.char_indices() {
        if escape {
            escape = false;
            continue;
        }
        if c == '\\' && in_string {
            escape = true;
            continue;
        }
        if c == '"' {
            in_string = !in_string;
            continue;
        }
        if !in_string && c == ':' {
            colon_pos = Some(i);
            break;
        }
    }

    let colon = colon_pos.ok_or_else(|| "Invalid JSON object: missing colon".to_string())?;
    let key_str = s[..colon].trim();
    let val_str = s[colon+1..].trim();

    if !(key_str.starts_with('"') && key_str.ends_with('"')) {
        return Err("Invalid JSON object: keys must be strings".to_string());
    }
    let key = key_str[1..key_str.len()-1].to_string();
    Ok((key, val_str.to_string()))
}

fn extract_int(v: &Value) -> Result<i64, String> {
    match v {
        Value::Int(n) => Ok(*n),
        v => Err(format!("expected int, got '{}'", v.type_name())),
    }
}

fn extract_iterable(v: &Value) -> Result<Vec<Value>, String> {
    match v {
        Value::List(l) => Ok(l.borrow().clone()),
        Value::Tuple(t) => Ok(t.clone()),
        Value::String(s) => Ok(s.chars().map(|c| Value::String(c.to_string())).collect()),
        v => Err(format!("'{}' object is not iterable", v.type_name())),
    }
}

fn add_values(a: &Value, b: &Value) -> Result<Value, String> {
    match (a, b) {
        (Value::Int(x), Value::Int(y)) => Ok(Value::Int(x + y)),
        (Value::Float(x), Value::Float(y)) => Ok(Value::Float(x + y)),
        (Value::Int(x), Value::Float(y)) => Ok(Value::Float(*x as f64 + y)),
        (Value::Float(x), Value::Int(y)) => Ok(Value::Float(x + *y as f64)),
        _ => Err(format!("unsupported operand type(s) for +: '{}' and '{}'", a.type_name(), b.type_name())),
    }
}

fn builtin_filter(compiler: &mut SchemaGenerator, args: Vec<Value>, _: HashMap<String, Value>) -> Result<Value, String> {
    if args.len() != 2 {
        return Err(format!("filter() takes exactly 2 arguments ({} given)", args.len()));
    }
    let func = &args[0];
    let iterable_value = &args[1];

    if iterable_value.is_dynamic() {
        let func_rc = match func {
            Value::Function(f) => f.clone(),
            _ => return Err("filter() requires a function as first argument for dynamic iterables".to_string()),
        };

        let item_name = "_filter_item";
        let predicate = compiler.generate_subplan_from_function(&func_rc, item_name)?;

        let op = SchemaOp::Filter {
            items: iterable_value.to_schema_value(),
            item_name: item_name.to_string(),
            predicate,
        };
        let id = compiler.add_schema_op(op);
        return Ok(Value::OpRef(id));
    }

    let iterable = extract_iterable(iterable_value)?;

    let mut result = Vec::new();
    for item in iterable {
        let test_result = compiler.call_value(func, vec![item.clone()], HashMap::new())?;
        if test_result.is_truthy() {
            result.push(item);
        }
    }

    Ok(Value::List(Rc::new(RefCell::new(result))))
}

fn builtin_map(compiler: &mut SchemaGenerator, args: Vec<Value>, _: HashMap<String, Value>) -> Result<Value, String> {
    if args.len() != 2 {
        return Err(format!("map() takes exactly 2 arguments ({} given)", args.len()));
    }
    let func = &args[0];
    let iterable_value = &args[1];

    if iterable_value.is_dynamic() {
        let func_rc = match func {
            Value::Function(f) => f.clone(),
            _ => return Err("map() requires a function as first argument for dynamic iterables".to_string()),
        };

        let item_name = "_map_item";
        let body = compiler.generate_subplan_from_function(&func_rc, item_name)?;

        let op = SchemaOp::Map {
            items: iterable_value.to_schema_value(),
            item_name: item_name.to_string(),
            body,
        };
        let id = compiler.add_schema_op(op);
        return Ok(Value::OpRef(id));
    }

    let iterable = extract_iterable(iterable_value)?;

    let mut result = Vec::new();
    for item in iterable {
        let mapped = compiler.call_value(func, vec![item], HashMap::new())?;
        result.push(mapped);
    }

    Ok(Value::List(Rc::new(RefCell::new(result))))
}
