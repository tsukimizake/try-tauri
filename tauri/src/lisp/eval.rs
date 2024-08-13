use crate::lisp::parser;
use std::collections::HashMap;

#[derive(Debug, Clone, PartialEq)]
enum Value {
    Integer(i64),
    Double(f64),
    Symbol(String),
    List(Vec<Value>),
    Function(fn(&[Value]) -> Result<Value, String>),
}

type Env = HashMap<String, Value>;

fn initial_env() -> Env {
    let mut env = Env::new();
    env.insert("+".to_string(), Value::Function(add));
    env
}

fn eval(expr: &parser::Expr, env: &mut Env) -> Result<Value, String> {
    match expr {
        parser::Expr::Symbol { name, .. } => env
            .get(name)
            .cloned()
            .ok_or_else(|| format!("Undefined symbol: {}", name)),
        parser::Expr::Integer { value, .. } => Ok(Value::Integer(*value)),
        parser::Expr::Double { value, .. } => Ok(Value::Double(*value)),
        parser::Expr::List { elements, .. } => {
            if elements.is_empty() {
                return Ok(Value::List(vec![]));
            }
            let first = eval(&elements[0], env)?;
            match first {
                Value::Function(f) => {
                    let args: Result<Vec<Value>, String> =
                        elements[1..].iter().map(|arg| eval(arg, env)).collect();
                    f(&args?)
                }
                _ => Err(format!("First element of list is not a function")),
            }
        }
    }
}

fn add(args: &[Value]) -> Result<Value, String> {
    if args.len() != 2 {
        return Err("add requires exactly 2 arguments".to_string());
    }
    match (&args[0], &args[1]) {
        (Value::Integer(a), Value::Integer(b)) => Ok(Value::Integer(a + b)),
        (Value::Double(a), Value::Double(b)) => Ok(Value::Double(a + b)),
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

        assert_eq!(eval(&expr, &mut env), Ok(Value::Integer(3)));
    }
}
