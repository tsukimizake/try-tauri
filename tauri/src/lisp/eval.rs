use crate::lisp::parser;
use crate::lisp::parser::Expr;
use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;

// env.rs
pub struct Env {
    parent: Option<Rc<RefCell<Env>>>,
    vars: HashMap<String, Rc<Expr>>,
}

impl Env {
    pub fn new() -> Env {
        Env {
            parent: None,
            vars: HashMap::new(),
        }
    }

    pub fn make_child(parent: Rc<RefCell<Env>>) -> Env {
        Env {
            parent: Some(parent),
            vars: HashMap::new(),
        }
    }
    pub fn insert(&mut self, name: String, value: Rc<Expr>) {
        self.vars.insert(name, value);
    }
    pub fn get(&self, name: &str) -> Option<Rc<Expr>> {
        self.vars.get(name).cloned().or_else(|| {
            self.parent
                .as_ref()
                .and_then(|parent| parent.borrow().get(name))
        })
    }
}

pub fn initial_env() -> Env {
    let mut env = Env::new();
    env.insert("+".to_string(), Rc::new(Expr::Builtin(add)));
    env
}

pub fn eval(expr: Rc<Expr>, env: &mut Env) -> Result<Rc<Expr>, String> {
    match expr.as_ref() {
        Expr::Symbol { name, .. } => env
            .get(name)
            .ok_or_else(|| format!("Undefined symbol: {}", name)),
        Expr::Integer { value, .. } => Ok(Rc::new(Expr::integer(*value))),
        Expr::Double { value, .. } => Ok(Rc::new(Expr::double(*value))),
        Expr::List { elements, .. } => eval_list(&elements[..], env),
        Expr::Quote { expr, .. } => Ok(Rc::new((**expr).clone())),
        Expr::Builtin(_) => Ok(expr),
        Expr::Lambda { .. } => Ok(expr),
    }
}

fn eval_list(elements: &[Rc<Expr>], env: &mut Env) -> Result<Rc<Expr>, String> {
    if elements.is_empty() {
        return Ok(Rc::new(Expr::list(vec![])));
    }
    let first = eval(elements[0].clone(), env)?;
    match &*first {
        Expr::Builtin(f) => {
            let args: Result<Vec<Rc<Expr>>, String> = elements[1..]
                .iter()
                .map(|arg| eval(arg.clone(), env))
                .collect();
            f(&args?)
        }
        _ => Err(format!("First element of list is not a function")),
    }
}

// (lambda (a b) (+ a b))
fn eval_lambda(expr: &Expr, env: &mut Env) -> Result<Rc<Expr>, String> {
    match expr {
        Expr::List { elements, .. } => {
            if elements.len() != 3 {
                return Err("lambda requires exactly 2 arguments".to_string());
            }
            let args = match &elements[1].as_ref() {
                Expr::List { elements, .. } => elements
                    .iter()
                    .map(|arg| match arg.as_ref() {
                        Expr::Symbol { name, .. } => Ok(name.clone()),
                        _ => Err("lambda requires a list of symbols as arguments".to_string()),
                    })
                    .collect::<Result<Vec<_>, _>>()?,
                _ => return Err("lambda requires a list as the first argument".to_string()),
            };

            let body = eval(elements[2].clone(), env)?;
            Ok(Rc::new(Expr::Lambda { args, body }))
        }
        _ => Err("lambda requires a list as an argument".to_string()),
    }
}

fn add(args: &[Rc<Expr>]) -> Result<Rc<Expr>, String> {
    args.iter()
        .try_fold(0, |acc, arg| match &**arg {
            Expr::Integer { value, .. } => Ok(acc + value),
            Expr::Double { value, .. } => Ok(acc + &(*value as i64)),
            _ => Err("add requires integer or double arguments".to_string()),
        })
        .map(|r| Rc::new(Expr::integer(r)))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_math() {
        let mut env = initial_env();

        let expr = parser::parse_expr("(+ 1 2 3)").unwrap();

        assert_eq!(eval(Rc::new(expr), &mut env), Ok(Rc::new(Expr::integer(6))));
    }
}
