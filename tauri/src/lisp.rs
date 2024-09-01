use std::sync::{Arc, Mutex};

use env::Env;

pub mod env;
pub mod eval;
pub mod parser;
pub type Expr = parser::Expr;

pub fn run_file(file: &str, env: Arc<Mutex<Env>>) -> Result<Arc<Expr>, String> {
    let exprs = parser::parse_file(file)?;
    eval::eval_exprs(exprs, env)
}
