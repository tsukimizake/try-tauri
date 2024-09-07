use std::collections::HashMap;
use std::sync::{Arc, Mutex};

use super::Expr;

#[derive(Debug)]
pub struct Env {
    parent: Option<Arc<Mutex<Env>>>,
    vars: HashMap<String, Arc<Expr>>,
    depth: usize,
}

impl Env {
    pub fn new() -> Env {
        Env {
            parent: None,
            vars: HashMap::new(),
            depth: 0,
        }
    }

    pub fn make_child(parent: Arc<Mutex<Env>>) -> Arc<Mutex<Env>> {
        Arc::new(Mutex::new(Env {
            parent: Some(parent.clone()),
            vars: HashMap::new(),
            depth: parent.lock().unwrap().depth + 1,
        }))
    }
    pub fn insert(&mut self, name: String, value: Arc<Expr>) {
        self.vars.insert(name, value);
    }
    pub fn get(&self, name: &str) -> Option<Arc<Expr>> {
        self.vars.get(name).cloned().or_else(|| {
            self.parent
                .as_ref()
                .and_then(|parent| parent.lock().unwrap().get(name))
        })
    }
}

impl PartialEq for Env {
    fn eq(&self, other: &Self) -> bool {
        self.vars == other.vars && self.depth == other.depth
    }
}
