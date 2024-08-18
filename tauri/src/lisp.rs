use std::rc::Rc;

mod eval;
mod parser;
pub type Expr = parser::Expr;

pub fn run_file(file: &str) -> Result<Rc<Expr>, String> {
    let env = eval::initial_env();
    let exprs = parser::parse_file(file)?;
    eval::eval_exprs(exprs, env)
}
