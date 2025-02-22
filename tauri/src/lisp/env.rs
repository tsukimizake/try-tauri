use std::collections::HashMap;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::{Arc, Mutex};
use truck_polymesh::PolygonMesh;

use super::gc;
use super::parser::Expr;

pub type PolyId = usize;

#[derive(Debug, Clone)]
pub struct StlObj {
    pub mesh: Arc<truck_polymesh::PolygonMesh>,
}

static COUNTER: AtomicUsize = AtomicUsize::new(1);

pub fn gen_id() -> usize {
    COUNTER.fetch_add(1, Ordering::Relaxed)
}

#[derive(Debug)]
pub struct Env {
    parent: Option<Arc<Mutex<Env>>>,
    vars: HashMap<String, Arc<Expr>>,
    depth: usize,
    polys: HashMap<PolyId, Arc<PolygonMesh>>,
    preview_list: Vec<PolyId>,
}

impl Env {
    pub fn collect_garbage(&mut self) {
        gc::collect_garbage(self);
    }
    pub fn new() -> Env {
        Env {
            parent: None,
            vars: HashMap::new(),
            depth: 0,
            polys: HashMap::new(),
            preview_list: Vec::new(),
        }
    }

    pub fn make_child(parent: Arc<Mutex<Env>>) -> Arc<Mutex<Env>> {
        Arc::new(Mutex::new(Env {
            parent: Some(parent.clone()),
            vars: HashMap::new(),
            depth: parent.lock().unwrap().depth + 1,
            polys: HashMap::new(),
            preview_list: Vec::new(),
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

    pub fn insert_stl(&mut self, mesh: Arc<truck_polymesh::PolygonMesh>) -> PolyId {
        let id = gen_id();
        self.polys.insert(id, mesh.clone().into());
        id
    }

    #[allow(dead_code)]
    pub fn get_stl(&self, id: PolyId) -> Option<Arc<StlObj>> {
        self.polys
            .get(&id)
            .map(|obj| {
                Arc::new(StlObj {
                    mesh: obj.clone().into(),
                })
            })
            .or_else(|| {
                self.parent
                    .as_ref()
                    .and_then(|parent| parent.lock().unwrap().get_stl(id))
            })
    }
    pub fn insert_preview_list(&mut self, id: PolyId) {
        self.preview_list.push(id);
    }

    pub fn polys(&self) -> Vec<(PolyId, Arc<PolygonMesh>)> {
        self.polys
            .iter()
            .map(|(id, obj)| (*id, obj.clone()))
            .collect()
    }

    pub fn preview_list(&self) -> Vec<PolyId> {
        self.preview_list.clone()
    }

    pub fn vars(&self) -> &HashMap<String, Arc<Expr>> {
        &self.vars
    }

    #[allow(dead_code)]
    pub fn vars_mut(&mut self) -> &mut HashMap<String, Arc<Expr>> {
        &mut self.vars
    }
    pub fn parent(&self) -> &Option<Arc<Mutex<Env>>> {
        &self.parent
    }

    pub fn retain_polys<F>(&mut self, mut f: F)
    where
        F: FnMut(&PolyId, &mut Arc<PolygonMesh>) -> bool,
    {
        self.polys.retain(|k, v| f(k, v));
    }
}

impl PartialEq for Env {
    fn eq(&self, other: &Self) -> bool {
        self.vars == other.vars && self.depth == other.depth
    }
}
