use std::collections::HashMap;
use std::sync::Arc;

use blueprint_engine_core::NativeFunction;

pub struct NativeModuleRegistry {
    modules: HashMap<String, HashMap<String, Arc<NativeFunction>>>,
}

impl NativeModuleRegistry {
    pub fn new() -> Self {
        Self {
            modules: HashMap::new(),
        }
    }

    pub fn register_module(&mut self, name: &str, functions: Vec<NativeFunction>) {
        let mut module_funcs = HashMap::new();
        for func in functions {
            module_funcs.insert(func.name.clone(), Arc::new(func));
        }
        self.modules.insert(name.to_string(), module_funcs);
    }

    pub fn get_module(&self, name: &str) -> Option<&HashMap<String, Arc<NativeFunction>>> {
        self.modules.get(name)
    }

    pub fn get_function(&self, module: &str, func: &str) -> Option<Arc<NativeFunction>> {
        self.modules.get(module).and_then(|m| m.get(func).cloned())
    }

    pub fn has_module(&self, name: &str) -> bool {
        self.modules.contains_key(name)
    }

    pub fn module_names(&self) -> Vec<&str> {
        self.modules.keys().map(|s| s.as_str()).collect()
    }
}

impl Default for NativeModuleRegistry {
    fn default() -> Self {
        Self::new()
    }
}
