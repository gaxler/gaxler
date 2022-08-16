use std::collections::HashMap;

use crate::Value;

#[derive(Debug, Clone)]
pub struct VarStore {
    store: HashMap<String, Value>
}


impl VarStore {
    pub fn new() -> Self {
        let store = HashMap::new();
        Self {store}
    }

    pub fn put(&mut self, ident_: String, val: Value) {
        self.store.insert(ident_, val);
    }

    pub fn get(&self, ident_: &str) -> Option<&Value> {
        self.store.get(ident_)
    }

    pub fn contains(&self, ident_: &str) -> bool {
        self.store.contains_key(ident_)
    }
}