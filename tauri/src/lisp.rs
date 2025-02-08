use std::sync::{Arc, Mutex};

use env::Env;

pub mod env;
pub mod eval;
pub mod parser;
pub type Expr = parser::Expr;
pub type Value = parser::Value;

pub fn run_file(file: &str, env: Arc<Mutex<Env>>) -> Result<Value, String> {
    let exprs = parser::parse_file(file)?;
    let result = eval::eval_exprs(exprs, env)?;
    Ok(parser::cast_evaled(result))
}
