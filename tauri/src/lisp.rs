mod eval;
mod parser;
pub type Expr = parser::Expr;

pub fn run_file(file: &str) -> Result<(), String> {
    let env = eval::initial_env();
    let exprs = parser::parse_file(file)?;
    let _result = eval::eval_exprs(exprs, env);
    Ok(())
}
