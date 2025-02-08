use elm_rs::{Elm, ElmDecode, ElmEncode};
use serde::Deserialize;
use serde::Serialize;
use std::collections::HashMap;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::{Arc, Mutex};

use super::super::elm::*;

use super::Expr;

pub type StlId = usize;

#[derive(Debug, Clone)]
pub struct StlObj {
    pub mesh: Arc<stl_io::IndexedMesh>,
}

#[derive(Serialize, Deserialize, Debug, Elm, ElmEncode, ElmDecode, Clone)]
pub struct StlObjSerde {
    pub mesh: SerdeIndexedMesh,
}

impl From<&StlObj> for StlObjSerde {
    fn from(obj: &StlObj) -> Self {
        StlObjSerde {
            mesh: (&*obj.mesh).into(),
        }
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
    stls: HashMap<StlId, StlObj>,
    preview_list: Vec<StlId>,
}

impl Env {
    pub fn new() -> Env {
        Env {
            parent: None,
            vars: HashMap::new(),
            depth: 0,
            stls: HashMap::new(),
            preview_list: Vec::new(),
        }
    }

    pub fn make_child(parent: Arc<Mutex<Env>>) -> Arc<Mutex<Env>> {
        Arc::new(Mutex::new(Env {
            parent: Some(parent.clone()),
            vars: HashMap::new(),
            depth: parent.lock().unwrap().depth + 1,
            stls: HashMap::new(),
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

    pub fn insert_stl(&mut self, mesh: Arc<stl_io::IndexedMesh>) -> StlId {
        let id = gen_id();
        self.stls.insert(id, StlObj { mesh });
        id
    }

    pub fn get_stl(&self, id: StlId) -> Option<Arc<StlObj>> {
        self.stls
            .get(&id)
            .map(|obj| {
                Arc::new(StlObj {
                    mesh: obj.mesh.clone(),
                })
            })
            .or_else(|| {
                self.parent
                    .as_ref()
                    .and_then(|parent| parent.lock().unwrap().get_stl(id))
            })
    }
    pub fn insert_preview_list(&mut self, id: StlId) {
        self.preview_list.push(id);
    }

    pub fn stls(&self) -> Vec<(StlId, StlObjSerde)> {
        self.stls
            .iter()
            .map(|(id, obj)| (*id, obj.into()))
            .collect()
    }

    pub fn preview_list(&self) -> Vec<StlId> {
        self.preview_list.clone()
    }
}

impl PartialEq for Env {
    fn eq(&self, other: &Self) -> bool {
        self.vars == other.vars && self.depth == other.depth
    }
}
