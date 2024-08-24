use elm_rs::{Elm, ElmDecode, ElmEncode};
use serde::Deserialize;
use serde::Serialize;

#[derive(Serialize, Deserialize, Debug, Elm, ElmEncode, ElmDecode, Clone)]
pub enum ToTauriCmdType {
    RequestStlFile(String),
    RequestCode(String),
}

#[derive(Serialize, Deserialize, Debug, Elm, ElmEncode, ElmDecode, Clone)]
pub enum FromTauriCmdType {
    StlBytes(Vec<u8>),
    Code(String),
}
