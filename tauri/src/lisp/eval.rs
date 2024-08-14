use crate::lisp::parser;
use crate::lisp::parser::Expr;
use std::collections::HashMap;

type Env = HashMap<String, Expr>;

pub fn initial_env() -> Env {
    let mut env = Env::new();
    env.insert("+".to_string(), Expr::Builtin(add));
    env
}

pub fn eval(expr: &Expr, env: &mut Env) -> Result<Expr, String> {
    match expr {
        Expr::Symbol { name, .. } => env
            .get(name)
            .cloned()
            .ok_or_else(|| format!("Undefined symbol: {}", name)),
        Expr::Integer { value, .. } => Ok(Expr::integer(*value)),
        Expr::Double { value, .. } => Ok(Expr::double(*value)),
        Expr::List { elements, .. } => {
            if elements.is_empty() {
                return Ok(Expr::list(vec![]));
            }
            let first = eval(&elements[0], env)?;
            match first {
                Expr::Builtin(f) => {
                    let args: Result<Vec<Expr>, String> =
                        elements[1..].iter().map(|arg| eval(arg, env)).collect();
                    f(&args?)
                }
                _ => Err(format!("First element of list is not a function")),
            }
        }
        Expr::QuotedList { elements, .. } => Ok(Expr::list(
            elements
                .into_iter()
                .map(|e| eval(e, env).unwrap())
                .collect(),
        )),
        Expr::Builtin(_) => Err("Cannot evaluate builtin function".to_string()),
    }
}

fn add(args: &[Expr]) -> Result<Expr, String> {
    if args.len() != 2 {
        return Err("add requires exactly 2 arguments".to_string());
    }
    match (&args[0], &args[1]) {
        (Expr::Integer { value: a, .. }, Expr::Integer { value: b, .. }) => {
            Ok(Expr::integer(a + b))
        }
        (Expr::Double { value: a, .. }, Expr::Double { value: b, .. }) => Ok(Expr::double(a + b)),
        _ => Err("add requires two integers or two doubles".to_string()),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_eval() {
        let mut env = initial_env();

        let expr = parser::run("(+ 1 2)").unwrap();

        assert_eq!(eval(&expr, &mut env), Ok(Expr::integer(3)));
    }
}
