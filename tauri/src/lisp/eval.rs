use crate::lisp::parser;
use crate::lisp::parser::Env;
use crate::lisp::parser::Expr;
use std::cell::RefCell;
use std::rc::Rc;

pub fn eval(expr: Rc<Expr>, env: Rc<RefCell<Env>>) -> Result<Rc<Expr>, String> {
    match expr.as_ref() {
        Expr::Symbol { name, .. } => env
            .borrow()
            .get(name)
            .ok_or_else(|| format!("Undefined symbol: {}", name)),
        Expr::Integer { value, .. } => Ok(Rc::new(Expr::integer(*value))),
        Expr::Double { value, .. } => Ok(Rc::new(Expr::double(*value))),
        Expr::List { elements, .. } => eval_list(&elements[..], env),
        Expr::Quote { expr, .. } => Ok(Rc::new((**expr).clone())),
        Expr::Builtin(_) => Ok(expr),
        Expr::Clausure { .. } => Ok(expr),
    }
}

fn eval_list(elements: &[Rc<Expr>], env: Rc<RefCell<Env>>) -> Result<Rc<Expr>, String> {
    if elements.is_empty() {
        return Ok(Rc::new(Expr::list(vec![])));
    }
    if elements[0].as_ref().is_symbol("lambda") {
        return eval_lambda(elements, env);
    }
    if elements[0].as_ref().is_symbol("define") {
        return eval_define(elements, env);
    }
    let first = eval(elements[0].clone(), env.clone())?;
    match &*first {
        Expr::Builtin(f) => {
            let args: Result<Vec<Rc<Expr>>, String> = elements[1..]
                .iter()
                .map(|arg| eval(arg.clone(), env.clone()))
                .collect();
            f(&args?)
        }
        Expr::Clausure { args, body, env } => {
            let newenv = Env::make_child(env.clone());
            for (arg, value) in args.iter().zip(elements.iter().skip(1)) {
                newenv
                    .borrow_mut()
                    .insert(arg.clone(), eval(value.clone(), env.clone())?);
            }
            eval(body.clone(), newenv.clone())
        }
        _ => Err(format!("First element of list is not a function")),
    }
}

fn eval_define(elements: &[Rc<Expr>], env: Rc<RefCell<Env>>) -> Result<Rc<Expr>, String> {
    todo!()
}

// (lambda (a b) (+ a b))
fn eval_lambda(expr: &[Rc<Expr>], env: Rc<RefCell<Env>>) -> Result<Rc<Expr>, String> {
    if expr.len() != 3 {
        return Err("lambda requires two arguments".to_string());
    }
    match (expr[1].as_ref(), expr[2].clone()) {
        (Expr::List { elements: args, .. }, body) => {
            let newenv = Env::make_child(env);
            let argnames: Vec<String> = args
                .iter()
                .map(|arg| {
                    arg.as_ref()
                        .as_symbol()
                        .expect("Lambda argument is not a symbol")
                        .to_string()
                })
                .collect();

            Ok(Rc::new(Expr::Clausure {
                args: argnames,
                body,
                env: newenv,
            }))
        }

        _ => Err("lambda requires a list as an argument".to_string()),
    }
}

pub fn initial_env() -> Rc<RefCell<Env>> {
    let mut env = Env::new();
    env.insert("+".to_string(), Rc::new(Expr::Builtin(add)));
    Rc::new(RefCell::new(env))
}

fn add(args: &[Rc<Expr>]) -> Result<Rc<Expr>, String> {
    args.iter()
        .try_fold(0, |acc, arg| match **arg {
            Expr::Integer { value, .. } => Ok(acc + value),
            Expr::Double { value, .. } => Ok(acc + value as i64),
            _ => Err("add requires integer or double arguments".to_string()),
        })
        .map(|r| Rc::new(Expr::integer(r)))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_math() {
        let env = initial_env();

        let expr = parser::parse_expr("(+ 1 2 3)").unwrap();

        assert_eq!(eval(Rc::new(expr), env), Ok(Rc::new(Expr::integer(6))));
    }
    #[test]
    fn test_lambda() {
        let env = initial_env();

        let expr = parser::parse_expr("((lambda (a b) (+ a b)) 1 2)").unwrap();
        let result = eval(Rc::new(expr), env.clone()).unwrap();
        assert_eq!(eval(result, env), Ok(Rc::new(Expr::integer(3))));
    }
}
