use crate::lisp::parser;
use crate::lisp::parser::Env;
use crate::lisp::parser::Expr;
use std::cell::RefCell;
use std::rc::Rc;

pub fn eval_exprs(exprs: Vec<parser::Expr>, env: Rc<RefCell<Env>>) -> Result<Rc<Expr>, String> {
    exprs
        .iter()
        .fold(Ok(Rc::new(Expr::list(vec![]))), |_, expr| {
            eval(Rc::new(expr.clone()), env.clone())
        })
}
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
            let args = &elements[1..];
            f(args, env)
        }
        Expr::Clausure {
            args,
            body,
            env: clausure_env,
        } => {
            let newenv = Env::make_child(clausure_env.clone());
            for (arg, value) in args.iter().zip(elements.iter().skip(1)) {
                let val = eval(value.clone(), env.clone());
                newenv.borrow_mut().insert(arg.clone(), val?);
            }

            eval(body.clone(), newenv.clone())
        }
        _ => Err(format!("First element of list is not a function")),
    }
}

// (define a 1) => (define a 1)
// (define (add a b) (+ a b)) => (define add (lambda (a b) (+ a b)))
// TODO: proper location and trailing_newline
fn eval_define(elements: &[Rc<Expr>], env: Rc<RefCell<Env>>) -> Result<Rc<Expr>, String> {
    match elements.get(1).map(|x| x.as_ref()) {
        Some(Expr::List {
            elements: fn_and_args,
            ..
        }) => {
            let define = elements[0].clone();
            let name = fn_and_args[0].clone();
            let fun = elements[1].clone();
            let args = fn_and_args[1..].to_vec();
            let lambda = Rc::new(Expr::List {
                elements: vec![
                    Rc::new(Expr::Symbol {
                        name: "lambda".to_string(),
                        location: fun.location(),
                        trailing_newline: false,
                    }),
                    Rc::new(Expr::List {
                        elements: args,
                        location: fun.location(),
                        trailing_newline: fun.has_newline(),
                    }),
                    elements[2].clone(),
                ],
                location: fun.location(),
                trailing_newline: fun.has_newline(),
            });
            eval_define_impl(&[define, name, lambda], env)
        }
        Some(_) => eval_define_impl(elements, env),
        None => Err("define requires a list or a symbol as an argument".to_string()),
    }
}

// (define a 1)
// (define add (lambda (a b) (+ a b)))
fn eval_define_impl(elements: &[Rc<Expr>], env: Rc<RefCell<Env>>) -> Result<Rc<Expr>, String> {
    if elements.len() != 3 {
        return Err("define requires two arguments".to_string());
    }
    match (elements[1].as_ref(), elements[2].clone()) {
        (Expr::Symbol { name, .. }, value) => {
            let value = eval(value, env.clone())?;
            env.borrow_mut().insert(name.clone(), value.clone());
            Ok(value)
        }
        (Expr::List { elements: args, .. }, body) => {
            let newenv = Env::make_child(env.clone());
            let argnames: Vec<String> = args
                .iter()
                .map(|arg| {
                    arg.as_ref()
                        .as_symbol()
                        .expect("Lambda argument is not a symbol")
                        .to_string()
                })
                .collect();

            let clausure = Rc::new(Expr::Clausure {
                args: argnames,
                body,
                env: newenv,
            });
            env.borrow_mut()
                .insert(elements[1].as_symbol()?.to_string(), clausure.clone());
            Ok(clausure)
        }
        _ => Err("define requires a symbol as an argument".to_string()),
    }
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
    env.insert("+".to_string(), Rc::new(Expr::Builtin(prim_add)));
    env.insert("-".to_string(), Rc::new(Expr::Builtin(prim_sub)));
    env.insert("<".to_string(), Rc::new(Expr::Builtin(prim_lessthan)));
    env.insert("if".to_string(), Rc::new(Expr::Builtin(prim_if)));
    Rc::new(RefCell::new(env))
}

fn prim_add(args: &[Rc<Expr>], env: Rc<RefCell<Env>>) -> Result<Rc<Expr>, String> {
    args.iter()
        .map(|arg| eval(arg.clone(), env.clone()).expect("add arg eval failed"))
        .try_fold(0, |acc, arg| match *arg {
            Expr::Integer { value, .. } => Ok(acc + value),
            Expr::Double { value, .. } => Ok(acc + value as i64),
            _ => Err("add requires integer or double arguments".to_string()),
        })
        .map(|r| Rc::new(Expr::integer(r)))
}

fn prim_sub(args: &[Rc<Expr>], env: Rc<RefCell<Env>>) -> Result<Rc<Expr>, String> {
    let head: Rc<Expr> = args
        .first()
        .ok_or("sub requires at least one argument".to_string())
        .and_then(|x| eval(x.clone(), env.clone()))?;
    let tail: Vec<Result<Rc<Expr>, String>> = args[1..]
        .iter()
        .map(|arg| eval(arg.clone(), env.clone()))
        .collect();
    let head = match head.as_ref() {
        Expr::Integer { value, .. } => *value,
        Expr::Double { value, .. } => *value as i64,
        _ => return Err("sub requires integer or double arguments".to_string()),
    };
    tail.iter()
        .try_fold(head, |acc, arg| match arg.clone()?.as_ref() {
            Expr::Integer { value, .. } => Ok(acc - value),
            Expr::Double { value, .. } => Ok(acc - *value as i64),
            _ => Err("sub requires integer or double arguments".to_string()),
        })
        .map(|r| Rc::new(Expr::integer(r)))
}

fn prim_lessthan(args: &[Rc<Expr>], env: Rc<RefCell<Env>>) -> Result<Rc<Expr>, String> {
    if args.len() != 2 {
        return Err("lessthan requires two arguments".to_string());
    }
    let evaled = eval_args(args, env)?;
    match (evaled[0].as_ref(), evaled[1].as_ref()) {
        (Expr::Integer { value: a, .. }, Expr::Integer { value: b, .. }) => {
            Ok(Rc::new(Expr::integer(if a < b { 1 } else { 0 })))
        }
        _ => Err("lessthan requires integer arguments".to_string()),
    }
}

fn eval_args(args: &[Rc<Expr>], env: Rc<RefCell<Env>>) -> Result<Vec<Rc<Expr>>, String> {
    args.iter()
        .map(|arg| eval(arg.clone(), env.clone()))
        .collect()
}

fn prim_if(args: &[Rc<Expr>], env: Rc<RefCell<Env>>) -> Result<Rc<Expr>, String> {
    if args.len() != 3 {
        return Err("if requires three arguments".to_string());
    }

    match eval(args[0].clone(), env.clone())?.as_ref() {
        Expr::Integer { value, .. } => {
            if *value != 0 {
                eval(args[1].clone(), env)
            } else {
                eval(args[2].clone(), env)
            }
        }
        _ => Err("First argument of if must be an integer".to_string()),
    }
}

#[cfg(test)]
mod tests {
    use std::collections::HashMap;

    use super::*;

    #[test]
    fn test_math() {
        let env = initial_env();
        let expr = parser::parse_expr("(+ 1 2 (+ 1 3))").unwrap();
        assert_eq!(eval(Rc::new(expr), env), Ok(Rc::new(Expr::integer(7))));
    }

    #[test]
    fn test_lambda() {
        let env = initial_env();
        let expr = parser::parse_expr("((lambda (a b) (+ a b)) 1 2)").unwrap();
        let result = eval(Rc::new(expr), env.clone());
        assert_eq!(result, Ok(Rc::new(Expr::integer(3))));
    }

    #[test]
    fn test_define() {
        let env = initial_env();
        let exprs = parser::parse_file("(define a 1) a").unwrap();
        let result = eval_exprs(exprs, env.clone());
        assert_eq!(result, Ok(Rc::new(Expr::integer(1))));
    }

    #[test]
    fn test_define_lambda1() {
        let env = initial_env();
        let exprs = parser::parse_file("(define add (lambda(a b) (+ a b))) (add 1 2)").unwrap();
        let result = eval_exprs(exprs, env.clone());
        assert_eq!(result, Ok(Rc::new(Expr::integer(3))));
    }

    #[test]
    fn test_define_lambda2() {
        let env = initial_env();
        let exprs = parser::parse_file("(define (add a b) (+ a b)) (add 1 2)").unwrap();
        let result = eval_exprs(exprs, env.clone());
        assert_eq!(result, Ok(Rc::new(Expr::integer(3))));
    }
    #[test]
    fn test_define_lambda3() {
        let env = initial_env();
        let exprs = parser::parse_file("(define (id a) a) (id 1)").unwrap();
        assert_eq!(
            eval_exprs(exprs, env.clone()),
            Ok(Rc::new(Expr::integer(1)))
        );
    }
    #[test]
    fn test_if() {
        let env = initial_env();
        let exprs = parser::parse_file("(if (< 1 2) 2 3)").unwrap();
        let result = eval_exprs(exprs, env.clone());
        assert_eq!(result, Ok(Rc::new(Expr::integer(2))));
    }
    #[test]
    fn test_if2() {
        let env = initial_env();
        let exprs = parser::parse_file("(if (< 2 1) 2 3)").unwrap();
        let result = eval_exprs(exprs, env.clone());
        assert_eq!(result, Ok(Rc::new(Expr::integer(3))));
    }
    #[test]
    fn test_if3() {
        let env = initial_env();
        let exprs = parser::parse_file("(if (< -3 1) 2 3)").unwrap();
        assert_eq!(
            eval_exprs(exprs, env.clone()),
            Ok(Rc::new(Expr::integer(2)))
        );
    }

    #[test]
    fn test_rec() {
        let env = initial_env();
        let exprs = parser::parse_file(
            "(define (fib n) (if (< n 2) n (+ (fib (- n 1)) (fib (- n 2))))) (fib 10)",
        )
        .unwrap();
        let result = eval_exprs(exprs, env.clone());
        assert_eq!(result, Ok(Rc::new(Expr::integer(55))));
    }
}
