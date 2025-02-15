use elm_rs::{Elm, ElmDecode, ElmEncode};
use serde::Deserialize;
use serde::Serialize;
use std::sync::Arc;
use truck_polymesh::stl::IntoStlIterator;
use truck_polymesh::stl::StlFace;
use truck_polymesh::PolygonMesh;

use crate::lisp::env::PolyId;

#[derive(Serialize, Deserialize, Debug, Elm, ElmEncode, ElmDecode, Clone)]
#[serde(tag = "t", content = "c")]
pub enum Value {
    Integer(i64),
    Double(f64),
    Stl(PolyId),
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
    pub polys: Vec<(PolyId, SerdeStlFaces)>,
    pub previews: Vec<PolyId>,
}

impl From<Arc<Evaled>> for Evaled {
    fn from(evaled: Arc<Evaled>) -> Evaled {
        Evaled {
            value: evaled.value.clone(),
            polys: evaled.polys.clone(),
            previews: evaled.previews.clone(),
        }
    }
}

// stl types

#[derive(Serialize, Deserialize, Debug, Elm, ElmEncode, ElmDecode, Clone)]
pub struct SerdeStlFaces(pub Vec<SerdeStlFace>);

#[derive(Serialize, Deserialize, Debug, Elm, ElmEncode, ElmDecode, Clone)]
pub struct SerdeStlFace([[f32; 3]; 3]); // removed normal vector

impl From<&Arc<PolygonMesh>> for SerdeStlFaces {
    fn from(mesh: &Arc<PolygonMesh>) -> SerdeStlFaces {
        let iter = mesh.into_iter();
        let mut res = Vec::new();
        for face in iter {
            res.push(SerdeStlFace::from(&face));
        }
        SerdeStlFaces(res)
    }
}

impl From<&StlFace> for SerdeStlFace {
    fn from(face: &StlFace) -> SerdeStlFace {
        SerdeStlFace(face.vertices)
    }
}

// msg types between tauri and elm

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
