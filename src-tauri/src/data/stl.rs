use elm_rs::{Elm, ElmDecode, ElmEncode};
use serde::Deserialize;
use serde::Serialize;

#[derive(Serialize, Deserialize, Debug, Elm, ElmEncode, ElmDecode)]
pub struct Stl {
    bytes: Vec<u8>,
}
