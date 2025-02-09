use elm_rs::{Elm, ElmDecode, ElmEncode};
use serde::Deserialize;
use serde::Serialize;
use std::sync::Arc;
use stl_io::{IndexedMesh, IndexedTriangle, Normal, Triangle, Vector, Vertex};

use crate::lisp::env::{StlId, StlObjSerde};

#[derive(Serialize, Deserialize, Debug, Elm, ElmEncode, ElmDecode, Clone)]
#[serde(tag = "t", content = "c")]
pub enum Value {
    Integer(i64),
    Double(f64),
    Stl(StlId),
    String(String),
    Symbol(String),
    List(Vec<Value>),
}

impl PartialEq for Value {
    fn eq(&self, other: &Self) -> bool {
        use Value::*;
        match (self, other) {
            (Integer(i1), Integer(i2)) => i1 == i2,
            (Double(d1), Double(d2)) => d1 == d2,
            (Stl(s1), Stl(s2)) => s1 == s2,
            (String(s1), String(s2)) => s1 == s2,
            (Symbol(s1), Symbol(s2)) => s1 == s2,
            (List(l1), List(l2)) => l1 == l2,
            _ => false,
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Elm, ElmEncode, ElmDecode, Clone)]
pub struct Evaled {
    pub value: Value,
    pub stls: Vec<(StlId, StlObjSerde)>,
    pub previews: Vec<StlId>,
}

impl From<Arc<Evaled>> for Evaled {
    fn from(evaled: Arc<Evaled>) -> Evaled {
        Evaled {
            value: evaled.value.clone(),
            stls: evaled.stls.clone(),
            previews: evaled.previews.clone(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, Elm, ElmEncode, ElmDecode)]
pub struct SerdeVector(pub [f32; 3]);

impl From<&Vector<f32>> for SerdeVector {
    fn from(v: &Vector<f32>) -> Self {
        SerdeVector((*v).into())
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, Elm, ElmEncode, ElmDecode)]
pub struct SerdeVertex(pub SerdeVector);

impl From<&Vertex> for SerdeVertex {
    fn from(v: &Vertex) -> Self {
        SerdeVertex(SerdeVector::from(v))
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, Elm, ElmEncode, ElmDecode)]
pub struct SerdeNormal(pub SerdeVector);

impl From<&Normal> for SerdeNormal {
    fn from(n: &Normal) -> Self {
        SerdeNormal(SerdeVector::from(n))
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, Elm, ElmEncode, ElmDecode)]
pub struct SerdeTriangle(pub SerdeNormal, pub [SerdeVertex; 3]);

impl From<&Triangle> for SerdeTriangle {
    fn from(t: &Triangle) -> Self {
        SerdeTriangle(
            SerdeNormal::from(&t.normal),
            [
                SerdeVertex::from(&t.vertices[0]),
                SerdeVertex::from(&t.vertices[1]),
                SerdeVertex::from(&t.vertices[2]),
            ]
        )
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, Elm, ElmEncode, ElmDecode)]
pub struct SerdeIndexedTriangle(pub SerdeVector, pub [usize; 3]);

impl From<&IndexedTriangle> for SerdeIndexedTriangle {
    fn from(triangle: &IndexedTriangle) -> Self {
        SerdeIndexedTriangle(
            (&triangle.normal).into(),
            triangle.vertices
        )
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, Elm, ElmEncode, ElmDecode)]
pub struct SerdeIndexedMesh {
    pub vertices: Vec<SerdeVector>,
    pub faces: Vec<SerdeIndexedTriangle>,
}

impl From<&IndexedMesh> for SerdeIndexedMesh {
    fn from(mesh: &IndexedMesh) -> Self {
        SerdeIndexedMesh {
            vertices: mesh.vertices.iter().map(|v| SerdeVector::from(v)).collect(),
            faces: mesh.faces.iter().map(|f| f.into()).collect(),
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Elm, ElmEncode, ElmDecode, Clone)]
#[serde(tag = "t", content = "c")]
pub enum ToTauriCmdType {
    // RequestStlFile(String),
    RequestCode(String),
    RequestEval,
}

#[derive(Serialize, Deserialize, Debug, Elm, ElmEncode, ElmDecode, Clone)]
#[serde(tag = "t", content = "c")]
pub enum FromTauriCmdType {
    // StlBytes(Vec<u8>),
    Code(String),
    EvalOk(Evaled),
    EvalError(String),
}
