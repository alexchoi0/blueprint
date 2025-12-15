use std::collections::HashMap;
use std::rc::Rc;
use std::cell::RefCell;
use super::value::Value;

#[derive(Debug, Clone)]
pub struct Scope {
    variables: Rc<RefCell<HashMap<String, Value>>>,
    parent: Option<Box<Scope>>,
}

impl Default for Scope {
    fn default() -> Self {
        Self::new()
    }
}

impl Scope {
    pub fn new() -> Self {
        Scope {
            variables: Rc::new(RefCell::new(HashMap::new())),
            parent: None,
        }
    }

    pub fn child(&self) -> Self {
        Scope {
            variables: Rc::new(RefCell::new(HashMap::new())),
            parent: Some(Box::new(self.clone())),
        }
    }

    pub fn get(&self, name: &str) -> Option<Value> {
        if let Some(value) = self.variables.borrow().get(name) {
            return Some(value.clone());
        }
        if let Some(parent) = &self.parent {
            return parent.get(name);
        }
        None
    }

    pub fn set(&self, name: impl Into<String>, value: Value) {
        self.variables.borrow_mut().insert(name.into(), value);
    }

    pub fn update(&self, name: &str, value: Value) -> bool {
        if self.variables.borrow().contains_key(name) {
            self.variables.borrow_mut().insert(name.to_string(), value);
            return true;
        }
        if let Some(parent) = &self.parent {
            return parent.update(name, value);
        }
        false
    }

    pub fn contains(&self, name: &str) -> bool {
        if self.variables.borrow().contains_key(name) {
            return true;
        }
        if let Some(parent) = &self.parent {
            return parent.contains(name);
        }
        false
    }

    pub fn get_all_globals(&self) -> HashMap<String, Value> {
        self.variables.borrow().clone()
    }
}
