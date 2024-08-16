use crate::lisp::parser;
use crate::lisp::parser::Expr;
use std::collections::HashMap;

type Env = HashMap<String, Box<Expr>>;

pub fn initial_env() -> Env {
    let mut env = Env::new();
    env.insert("+".to_string(), Box::new(Expr::Builtin(add)));
    env
}

pub fn eval(expr: &Expr, env: &mut Env) -> Result<Box<Expr>, String> {
    match expr {
        Expr::Symbol { name, .. } => env
            .get(name)
            .cloned()
            .ok_or_else(|| format!("Undefined symbol: {}", name)),
        Expr::Integer { value, .. } => Ok(Box::new(Expr::integer(*value))),
        Expr::Double { value, .. } => Ok(Box::new(Expr::double(*value))),
        Expr::List { elements, .. } => {
            if elements.is_empty() {
                return Ok(Box::new(Expr::list(vec![])));
            }
            let first = eval(&elements[0], env)?;
            match *first {
                Expr::Builtin(f) => {
                    let args: Result<Vec<Expr>, String> = elements[1..]
                        .iter()
                        .map(|arg| eval(arg, env).map(|r| *r))
                        .collect();
                    f(&args?)
                }
                _ => Err(format!("First element of list is not a function")),
            }
        }
        Expr::Quote { expr, .. } => Ok((*expr).clone()),
        Expr::Builtin(_) => Err("Cannot evaluate builtin function".to_string()),
    }
}

// (def (add a b) (+ a b))
fn eval_def(expr: &Expr, env: &mut Env) -> Result<Box<Expr>, String> {
    match expr {
        Expr::List { elements, .. } => {
            if elements.len() != 3 {
                return Err("def requires exactly 2 arguments".to_string());
            }
            let (name, args) = match &elements[1] {
                Expr::Symbol { name, .. } => (name, vec![]),
                Expr::List { elements, .. } => {
                    let mut iter = elements.iter();
                    let name = match iter.next() {
                        Some(Expr::Symbol { name, .. }) => name,
                        _ => {
                            return Err("def requires a list starting with a symbol as arguments"
                                .to_string())
                        }
                    };
                    let args = iter
                        .map(|arg| match arg {
                            Expr::Symbol { name, .. } => Ok(name.clone()),
                            _ => Err("def requires a list of symbols as arguments".to_string()),
                        })
                        .collect::<Result<Vec<_>, _>>()?;
                    (name, args)
                }
                _ => return Err("def requires a symbol or list as the first argument".to_string()),
            };

            let value = eval(&elements[2], env)?;
            env.insert(name.clone(), value);
            Ok(Box::new(Expr::list(vec![])))
        }
        _ => Err("def requires a list as an argument".to_string()),
    }
}

fn add(args: &[Expr]) -> Result<Box<Expr>, String> {
    if args.len() != 2 {
        return Err("add requires exactly 2 arguments".to_string());
    }
    match (&args[0], &args[1]) {
        (Expr::Integer { value: a, .. }, Expr::Integer { value: b, .. }) => {
            Ok(Box::new(Expr::integer(a + b)))
        }
        (Expr::Double { value: a, .. }, Expr::Double { value: b, .. }) => {
            Ok(Box::new(Expr::double(a + b)))
        }
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

        assert_eq!(eval(&expr, &mut env), Ok(Box::new(Expr::integer(3))));
    }
}
