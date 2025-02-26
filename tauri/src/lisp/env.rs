use std::collections::HashMap;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::{Arc, Mutex};
use truck_polymesh::PolygonMesh;

use super::gc;
use super::parser::Expr;

pub type ModelId = usize;

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub enum Model {
    Vertex(Arc<truck_modeling::Vertex>),
    Edge(Arc<truck_modeling::Edge>),
    Wire(Arc<truck_modeling::Wire>),
    Face(Arc<truck_modeling::Face>),
    Shell(Arc<truck_modeling::Shell>),
    Solid(Arc<truck_modeling::Solid>),
    Mesh(Arc<truck_polymesh::PolygonMesh>),
}

impl From<Arc<truck_modeling::Vertex>> for Model {
    fn from(vertex: Arc<truck_modeling::Vertex>) -> Self {
        Model::Vertex(vertex)
    }
}

impl From<Arc<truck_modeling::Edge>> for Model {
    fn from(edge: Arc<truck_modeling::Edge>) -> Self {
        Model::Edge(edge)
    }
}

impl From<Arc<truck_modeling::Wire>> for Model {
    fn from(wire: Arc<truck_modeling::Wire>) -> Self {
        Model::Wire(wire)
    }
}

impl From<Arc<truck_modeling::Face>> for Model {
    fn from(face: Arc<truck_modeling::Face>) -> Self {
        Model::Face(face)
    }
}

impl From<Arc<truck_modeling::Shell>> for Model {
    fn from(shell: Arc<truck_modeling::Shell>) -> Self {
        Model::Shell(shell)
    }
}

impl From<Arc<truck_modeling::Solid>> for Model {
    fn from(solid: Arc<truck_modeling::Solid>) -> Self {
        Model::Solid(solid)
    }
}

impl From<Arc<truck_polymesh::PolygonMesh>> for Model {
    fn from(mesh: Arc<truck_polymesh::PolygonMesh>) -> Self {
        Model::Mesh(mesh)
    }
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
    models: HashMap<ModelId, Arc<Model>>,
    preview_list: Vec<ModelId>,
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
            models: HashMap::new(),
            preview_list: Vec::new(),
        }
    }

    pub fn make_child(parent: &Arc<Mutex<Env>>) -> Arc<Mutex<Env>> {
        Arc::new(Mutex::new(Env {
            parent: Some(parent.clone()),
            vars: HashMap::new(),
            depth: parent.lock().unwrap().depth + 1,
            models: HashMap::new(),
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

    pub fn insert_model<T: Into<Model>>(&mut self, model_into: T) -> ModelId {
        let model = model_into.into();
        let id = gen_id();
        self.models.insert(id, Arc::new(model));
        id
    }

    #[allow(dead_code)]
    pub fn get_model(&self, id: ModelId) -> Option<Arc<Model>> {
        self.models.get(&id).cloned().or_else(|| {
            self.parent
                .as_ref()
                .and_then(|parent| parent.lock().unwrap().get_model(id))
        })
    }
    pub fn insert_preview_list(&mut self, id: ModelId) {
        self.preview_list.push(id);
    }

    pub fn polys(&self) -> Vec<(ModelId, Arc<PolygonMesh>)> {
        self.models
            .iter()
            .filter_map(|(id, model)| {
                if let Model::Mesh(mesh) = model.as_ref() {
                    Some((*id, mesh.clone()))
                } else {
                    None
                }
            })
            .collect()
    }

    pub fn preview_list(&self) -> Vec<ModelId> {
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
        F: FnMut(&ModelId, &mut Arc<Model>) -> bool,
    {
        self.models.retain(|k, v| f(k, v));
    }
}

impl PartialEq for Env {
    fn eq(&self, other: &Self) -> bool {
        self.vars == other.vars && self.depth == other.depth
    }
}

inventory::collect!(LispPrimitive);

#[doc(hidden)]
pub(crate) struct LispPrimitive {
    pub name: &'static str,
    pub func: fn(&[Arc<Expr>], Arc<Mutex<crate::lisp::env::Env>>) -> Result<Arc<Expr>, String>,
}
