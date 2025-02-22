use std::sync::{Arc, Mutex};

use env::Env;

use crate::elm::Evaled;

pub mod env;
pub mod eval;
mod gc;
pub mod parser;

pub fn run_file(file: &str, env: Arc<Mutex<Env>>) -> Result<Arc<Evaled>, String> {
    let exprs = parser::parse_file(file)?;
    eval::eval_exprs(exprs, env)
}
