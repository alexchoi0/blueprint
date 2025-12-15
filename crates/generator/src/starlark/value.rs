use std::collections::{HashMap, BTreeMap, HashSet};
use std::fmt;
use std::rc::Rc;
use std::cell::RefCell;
use std::hash::{Hash, Hasher};
use blueprint_common::{SchemaOpId, SchemaValue, RecordedValue};

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum HashableValue {
    None,
    Bool(bool),
    Int(i64),
    String(String),
    Tuple(Vec<HashableValue>),
}

impl Hash for HashableValue {
    fn hash<H: Hasher>(&self, state: &mut H) {
        std::mem::discriminant(self).hash(state);
        match self {
            HashableValue::None => {}
            HashableValue::Bool(b) => b.hash(state),
            HashableValue::Int(n) => n.hash(state),
            HashableValue::String(s) => s.hash(state),
            HashableValue::Tuple(t) => t.hash(state),
        }
    }
}

impl HashableValue {
    pub fn from_value(v: &Value) -> Result<Self, String> {
        match v {
            Value::None => Ok(HashableValue::None),
            Value::Bool(b) => Ok(HashableValue::Bool(*b)),
            Value::Int(n) => Ok(HashableValue::Int(*n)),
            Value::String(s) => Ok(HashableValue::String(s.clone())),
            Value::Tuple(t) => {
                let items: Result<Vec<HashableValue>, String> = t.iter()
                    .map(HashableValue::from_value)
                    .collect();
                Ok(HashableValue::Tuple(items?))
            }
            _ => Err(format!("unhashable type: '{}'", v.type_name())),
        }
    }

    pub fn to_value(&self) -> Value {
        match self {
            HashableValue::None => Value::None,
            HashableValue::Bool(b) => Value::Bool(*b),
            HashableValue::Int(n) => Value::Int(*n),
            HashableValue::String(s) => Value::String(s.clone()),
            HashableValue::Tuple(t) => Value::Tuple(t.iter().map(|h| h.to_value()).collect()),
        }
    }
}

#[derive(Clone)]
pub enum Value {
    None,
    Bool(bool),
    Int(i64),
    Float(f64),
    String(String),
    Bytes(Vec<u8>),
    List(Rc<RefCell<Vec<Value>>>),
    Dict(Rc<RefCell<HashMap<String, Value>>>),
    Set(Rc<RefCell<HashSet<HashableValue>>>),
    Tuple(Vec<Value>),
    Function(Rc<Function>),
    BuiltinFunction(Rc<dyn Fn(&mut super::generator::SchemaGenerator, Vec<Value>, HashMap<String, Value>) -> Result<Value, String>>),
    OpRef(SchemaOpId),
    ParamRef(String),
    Struct(HashMap<String, Value>),
    Partial {
        func: Rc<Function>,
        bound_args: Vec<Value>,
        bound_kwargs: HashMap<String, Value>,
    },
}

impl fmt::Debug for Value {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Value::None => write!(f, "None"),
            Value::Bool(b) => write!(f, "Bool({:?})", b),
            Value::Int(n) => write!(f, "Int({:?})", n),
            Value::Float(n) => write!(f, "Float({:?})", n),
            Value::String(s) => write!(f, "String({:?})", s),
            Value::Bytes(b) => write!(f, "Bytes({:?})", b),
            Value::List(l) => write!(f, "List({:?})", l.borrow()),
            Value::Dict(d) => write!(f, "Dict({:?})", d.borrow()),
            Value::Set(s) => write!(f, "Set({:?})", s.borrow()),
            Value::Tuple(t) => write!(f, "Tuple({:?})", t),
            Value::Function(func) => write!(f, "Function({:?})", func),
            Value::BuiltinFunction(_) => write!(f, "BuiltinFunction(<fn>)"),
            Value::OpRef(id) => write!(f, "OpRef({:?})", id),
            Value::ParamRef(name) => write!(f, "ParamRef({:?})", name),
            Value::Struct(fields) => write!(f, "Struct({:?})", fields),
            Value::Partial { func, bound_args, bound_kwargs } => {
                write!(f, "Partial({:?}, args={:?}, kwargs={:?})", func.name, bound_args, bound_kwargs)
            }
        }
    }
}

pub type BuiltinFn = Rc<dyn Fn(&mut super::generator::SchemaGenerator, Vec<Value>, HashMap<String, Value>) -> Result<Value, String>>;

#[derive(Debug, Clone)]
pub struct Function {
    pub name: String,
    pub params: Vec<Parameter>,
    pub body: FunctionBody,
    pub closure: super::scope::Scope,
}

#[derive(Debug, Clone)]
pub struct Parameter {
    pub name: String,
    pub default: Option<Value>,
    pub is_args: bool,
    pub is_kwargs: bool,
}

#[derive(Clone)]
pub enum FunctionBody {
    Ast(Box<starlark_syntax::syntax::ast::AstStmt>),
    Lambda(Box<starlark_syntax::syntax::ast::AstExpr>),
}

impl fmt::Debug for FunctionBody {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            FunctionBody::Ast(_) => write!(f, "FunctionBody::Ast(...)"),
            FunctionBody::Lambda(_) => write!(f, "FunctionBody::Lambda(...)"),
        }
    }
}

impl Value {
    pub fn is_truthy(&self) -> bool {
        match self {
            Value::None => false,
            Value::Bool(b) => *b,
            Value::Int(n) => *n != 0,
            Value::Float(f) => *f != 0.0,
            Value::String(s) => !s.is_empty(),
            Value::Bytes(b) => !b.is_empty(),
            Value::List(l) => !l.borrow().is_empty(),
            Value::Dict(d) => !d.borrow().is_empty(),
            Value::Set(s) => !s.borrow().is_empty(),
            Value::Tuple(t) => !t.is_empty(),
            Value::Function(_) => true,
            Value::BuiltinFunction(_) => true,
            Value::OpRef(_) => true,
            Value::ParamRef(_) => true,
            Value::Struct(fields) => !fields.is_empty(),
            Value::Partial { .. } => true,
        }
    }

    pub fn type_name(&self) -> &'static str {
        match self {
            Value::None => "NoneType",
            Value::Bool(_) => "bool",
            Value::Int(_) => "int",
            Value::Float(_) => "float",
            Value::String(_) => "string",
            Value::Bytes(_) => "bytes",
            Value::List(_) => "list",
            Value::Dict(_) => "dict",
            Value::Set(_) => "set",
            Value::Tuple(_) => "tuple",
            Value::Function(_) => "function",
            Value::BuiltinFunction(_) => "builtin_function",
            Value::OpRef(_) => "op_ref",
            Value::ParamRef(_) => "param_ref",
            Value::Struct(_) => "struct",
            Value::Partial { .. } => "partial",
        }
    }

    pub fn contains_dynamic(&self) -> bool {
        match self {
            Value::OpRef(_) => true,
            Value::ParamRef(_) => true,
            Value::List(l) => l.borrow().iter().any(|v| v.contains_dynamic()),
            Value::Tuple(t) => t.iter().any(|v| v.contains_dynamic()),
            Value::Dict(d) => d.borrow().values().any(|v| v.contains_dynamic()),
            Value::Set(s) => s.borrow().iter().any(|h| h.to_value().contains_dynamic()),
            _ => false,
        }
    }

    pub fn is_op_ref(&self) -> bool {
        matches!(self, Value::OpRef(_))
    }

    pub fn is_dynamic(&self) -> bool {
        matches!(self, Value::OpRef(_) | Value::ParamRef(_))
    }

    pub fn to_schema_value(&self) -> SchemaValue {
        match self {
            // Primitives
            Value::None => SchemaValue::Literal(RecordedValue::None),
            Value::Bool(b) => SchemaValue::Literal(RecordedValue::Bool(*b)),
            Value::Int(n) => SchemaValue::Literal(RecordedValue::Int(*n)),
            Value::Float(f) => SchemaValue::Literal(RecordedValue::Float(*f)),
            Value::String(s) => SchemaValue::Literal(RecordedValue::String(s.clone())),
            Value::Bytes(b) => Self::bytes_to_schema_value(b),

            // Collections
            Value::List(l) => Self::list_to_schema_value(l),
            Value::Dict(d) => Self::dict_to_schema_value(d),
            Value::Set(s) => Self::set_to_schema_value(s),
            Value::Tuple(t) => Self::tuple_to_schema_value(t),
            Value::Struct(fields) => Self::struct_to_schema_value(fields),

            // Dynamic references
            Value::OpRef(id) => SchemaValue::OpRef { id: *id, path: Vec::new() },
            Value::ParamRef(name) => SchemaValue::ParamRef(name.clone()),

            // Non-serializable
            Value::Function(_) | Value::BuiltinFunction(_) | Value::Partial { .. } => {
                SchemaValue::Literal(RecordedValue::None)
            }
        }
    }

    pub fn to_literal(&self) -> Option<RecordedValue> {
        match self.to_schema_value() {
            SchemaValue::Literal(rv) => Some(rv),
            _ => None,
        }
    }

    fn bytes_to_schema_value(bytes: &[u8]) -> SchemaValue {
        let items: Vec<RecordedValue> = bytes.iter()
            .map(|byte| RecordedValue::Int(*byte as i64))
            .collect();
        SchemaValue::Literal(RecordedValue::List(items))
    }

    fn list_to_schema_value(list: &Rc<RefCell<Vec<Value>>>) -> SchemaValue {
        let borrowed = list.borrow();
        if borrowed.iter().any(|v| v.contains_dynamic()) {
            SchemaValue::List(borrowed.iter().map(|v| v.to_schema_value()).collect())
        } else {
            let items: Vec<RecordedValue> = borrowed.iter()
                .filter_map(|v| v.to_literal())
                .collect();
            SchemaValue::Literal(RecordedValue::List(items))
        }
    }

    fn dict_to_schema_value(dict: &Rc<RefCell<HashMap<String, Value>>>) -> SchemaValue {
        let items: BTreeMap<String, RecordedValue> = dict.borrow().iter()
            .filter_map(|(k, v)| v.to_literal().map(|rv| (k.clone(), rv)))
            .collect();
        SchemaValue::Literal(RecordedValue::Dict(items))
    }

    fn set_to_schema_value(set: &Rc<RefCell<HashSet<HashableValue>>>) -> SchemaValue {
        let items: Vec<RecordedValue> = set.borrow().iter()
            .filter_map(|h| h.to_value().to_literal())
            .collect();
        SchemaValue::Literal(RecordedValue::List(items))
    }

    fn tuple_to_schema_value(tuple: &[Value]) -> SchemaValue {
        let items: Vec<RecordedValue> = tuple.iter()
            .filter_map(|v| v.to_literal())
            .collect();
        SchemaValue::Literal(RecordedValue::List(items))
    }

    fn struct_to_schema_value(fields: &HashMap<String, Value>) -> SchemaValue {
        let items: BTreeMap<String, RecordedValue> = fields.iter()
            .filter_map(|(k, v)| v.to_literal().map(|rv| (k.clone(), rv)))
            .collect();
        SchemaValue::Literal(RecordedValue::Dict(items))
    }

    pub fn to_string_repr(&self) -> String {
        match self {
            Value::None => "None".to_string(),
            Value::Bool(true) => "True".to_string(),
            Value::Bool(false) => "False".to_string(),
            Value::Int(n) => n.to_string(),
            Value::Float(f) => format!("{}", f),
            Value::String(s) => s.clone(),
            Value::Bytes(b) => {
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
            Value::List(l) => {
                let items: Vec<String> = l.borrow().iter().map(|v| v.to_repr()).collect();
                format!("[{}]", items.join(", "))
            }
            Value::Dict(d) => {
                let items: Vec<String> = d.borrow().iter()
                    .map(|(k, v)| format!("\"{}\": {}", k, v.to_repr()))
                    .collect();
                format!("{{{}}}", items.join(", "))
            }
            Value::Set(s) => {
                if s.borrow().is_empty() {
                    "set()".to_string()
                } else {
                    let items: Vec<String> = s.borrow().iter()
                        .map(|h| h.to_value().to_repr())
                        .collect();
                    format!("set([{}])", items.join(", "))
                }
            }
            Value::Tuple(t) => {
                let items: Vec<String> = t.iter().map(|v| v.to_repr()).collect();
                if t.len() == 1 {
                    format!("({},)", items[0])
                } else {
                    format!("({})", items.join(", "))
                }
            }
            Value::Function(f) => format!("<function {}>", f.name),
            Value::BuiltinFunction(_) => "<builtin_function>".to_string(),
            Value::OpRef(id) => format!("<op_ref {}>", id.0),
            Value::ParamRef(name) => format!("<param_ref {}>", name),
            Value::Struct(fields) => {
                let items: Vec<String> = fields.iter()
                    .map(|(k, v)| format!("{}={}", k, v.to_repr()))
                    .collect();
                format!("struct({})", items.join(", "))
            }
            Value::Partial { func, .. } => format!("<partial {}>", func.name),
        }
    }

    pub fn to_repr(&self) -> String {
        match self {
            Value::String(s) => format!("\"{}\"", s),
            _ => self.to_string_repr(),
        }
    }
}

impl PartialEq for Value {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Value::None, Value::None) => true,
            (Value::Bool(a), Value::Bool(b)) => a == b,
            (Value::Int(a), Value::Int(b)) => a == b,
            (Value::Float(a), Value::Float(b)) => a == b,
            (Value::Int(a), Value::Float(b)) => (*a as f64) == *b,
            (Value::Float(a), Value::Int(b)) => *a == (*b as f64),
            (Value::String(a), Value::String(b)) => a == b,
            (Value::Bytes(a), Value::Bytes(b)) => a == b,
            (Value::Tuple(a), Value::Tuple(b)) => a == b,
            (Value::OpRef(a), Value::OpRef(b)) => a == b,
            (Value::ParamRef(a), Value::ParamRef(b)) => a == b,
            _ => false,
        }
    }
}

impl PartialOrd for Value {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        match (self, other) {
            (Value::Int(a), Value::Int(b)) => a.partial_cmp(b),
            (Value::Float(a), Value::Float(b)) => a.partial_cmp(b),
            (Value::Int(a), Value::Float(b)) => (*a as f64).partial_cmp(b),
            (Value::Float(a), Value::Int(b)) => a.partial_cmp(&(*b as f64)),
            (Value::String(a), Value::String(b)) => a.partial_cmp(b),
            (Value::Bytes(a), Value::Bytes(b)) => a.partial_cmp(b),
            (Value::Tuple(a), Value::Tuple(b)) => {
                for (av, bv) in a.iter().zip(b.iter()) {
                    match av.partial_cmp(bv) {
                        Some(std::cmp::Ordering::Equal) => continue,
                        other => return other,
                    }
                }
                a.len().partial_cmp(&b.len())
            }
            _ => None,
        }
    }
}
