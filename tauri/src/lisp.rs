use std::sync::{Arc, Mutex};

use env::Env;

pub mod env;
pub mod eval;
pub mod parser;
pub type Expr = parser::Expr;
pub type Value = parser::Value;
pub type Evaled = super::elm::Evaled;

pub fn run_file(file: &str, env: Arc<Mutex<Env>>) -> Result<Arc<Evaled>, String> {
    let exprs = parser::parse_file(file)?;
    eval::eval_exprs(exprs, env)
}
