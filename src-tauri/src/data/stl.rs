use elm_rs::{Elm, ElmDecode, ElmEncode};
use serde::Deserialize;
use serde::Serialize;

#[derive(Serialize, Deserialize, Debug, Elm, ElmEncode, ElmDecode, Clone)]
pub struct Stl {
    pub bytes: Vec<u8>,
}
