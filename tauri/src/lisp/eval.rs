use crate::lisp::env::{Env, LispPrimitive};
use crate::lisp::parser;
use crate::lisp::parser::Expr;
use inventory;
use lisp_macro::lisp_fn;
use std::sync::{Arc, Mutex};

use super::Evaled;

pub fn eval_exprs(exprs: Vec<parser::Expr>, env: Arc<Mutex<Env>>) -> Result<Arc<Evaled>, String> {
    let evaled_expr = exprs
        .iter()
        .fold(Ok(Arc::new(Expr::list(vec![]))), |_, expr| {
            eval(Arc::new(expr.clone()), env.clone())
        });
    let polys = env
        .lock()
        .unwrap()
        .polys()
        .iter()
        .map(|(id, poly)| -> (usize, crate::elm_interface::SerdeStlFaces) { (*id, poly.into()) })
        .collect();
    let previews = env.lock().unwrap().preview_list();
    evaled_expr.map(|expr| {
        Arc::new(Evaled {
            value: parser::cast_evaled(expr),
            polys,
            previews,
        })
    })
}

pub fn eval(expr: Arc<Expr>, env: Arc<Mutex<Env>>) -> Result<Arc<Expr>, String> {
    match expr.as_ref() {
        Expr::Symbol { name, .. } => env
            .lock()
            .unwrap()
            .get(name)
            .ok_or_else(|| format!("Undefined symbol: {}", name)),
        Expr::Integer { value, .. } => Ok(Arc::new(Expr::integer(*value))),
        Expr::Double { value, .. } => Ok(Arc::new(Expr::double(*value))),
        Expr::List { elements, .. } => eval_list(&elements[..], env),
        Expr::String { value, .. } => Ok(Arc::new(Expr::string(value.clone()))),
        Expr::Stl { id, .. } => Ok(Arc::new(Expr::stl(*id))),
        Expr::Quote { expr, .. } => Ok(Arc::new((**expr).clone())),
        Expr::Builtin { .. } => Ok(expr),
        Expr::Clausure { .. } => Ok(expr),
    }
}

fn eval_list(elements: &[Arc<Expr>], env: Arc<Mutex<Env>>) -> Result<Arc<Expr>, String> {
    if elements.is_empty() {
        return Ok(Arc::new(Expr::list(vec![])));
    }
    if elements[0].as_ref().is_symbol("lambda") {
        return eval_lambda(elements, env);
    }
    if elements[0].as_ref().is_symbol("define") {
        return eval_define(elements, env);
    }
    let first = eval(elements[0].clone(), env.clone())?;
    match &*first {
        Expr::Builtin { fun, .. } => {
            let args = &elements[1..];
            fun(args, env)
        }
        Expr::Clausure {
            args,
            body,
            env: clausure_env,
        } => {
            let newenv = Env::make_child(clausure_env.clone());
            for (arg, value) in args.iter().zip(elements.iter().skip(1)) {
                let val = eval(value.clone(), env.clone());
                newenv.lock().unwrap().insert(arg.clone(), val?);
            }

            eval(body.clone(), newenv.clone())
        }
        _ => Err(format!("First element of list is not a function")),
    }
}

// (define a 1) => (define a 1)
// (define (add a b) (+ a b)) => (define add (lambda (a b) (+ a b)))
// TODO: proper location and trailing_newline
fn eval_define(elements: &[Arc<Expr>], env: Arc<Mutex<Env>>) -> Result<Arc<Expr>, String> {
    match elements.get(1).map(|x| x.as_ref()) {
        Some(Expr::List {
            elements: fn_and_args,
            ..
        }) => {
            let define = elements[0].clone();
            let name = fn_and_args[0].clone();
            let fun = elements[1].clone();
            let args = fn_and_args[1..].to_vec();
            let lambda = Arc::new(Expr::List {
                elements: vec![
                    Arc::new(Expr::Symbol {
                        name: "lambda".to_string(),
                        location: fun.location(),
                        trailing_newline: false,
                    }),
                    Arc::new(Expr::List {
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
fn eval_define_impl(elements: &[Arc<Expr>], env: Arc<Mutex<Env>>) -> Result<Arc<Expr>, String> {
    if elements.len() != 3 {
        return Err("define requires two arguments".to_string());
    }
    match (elements[1].as_ref(), elements[2].clone()) {
        (Expr::Symbol { name, .. }, value) => {
            let value = eval(value, env.clone())?;
            env.lock().unwrap().insert(name.clone(), value.clone());
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

            let clausure = Arc::new(Expr::Clausure {
                args: argnames,
                body,
                env: newenv,
            });
            env.lock()
                .unwrap()
                .insert(elements[1].as_symbol()?.to_string(), clausure.clone());
            Ok(clausure)
        }
        _ => Err("define requires a symbol as an argument".to_string()),
    }
}

// (lambda (a b) (+ a b))
fn eval_lambda(expr: &[Arc<Expr>], env: Arc<Mutex<Env>>) -> Result<Arc<Expr>, String> {
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

            Ok(Arc::new(Expr::Clausure {
                args: argnames,
                body,
                env: newenv,
            }))
        }

        _ => Err("lambda requires a list as an argument".to_string()),
    }
}

pub fn default_env() -> Env {
    let mut env = Env::new();

    // Register all primitives that used the lisp_fn macro
    for primitive in inventory::iter::<LispPrimitive> {
        env.insert(
            primitive.name.to_string(),
            Arc::new(Expr::Builtin {
                name: primitive.name.to_string(),
                fun: primitive.func,
            }),
        );
    }

    env
}

#[lisp_fn("+")]
fn prim_add(args: &[Arc<Expr>], env: Arc<Mutex<Env>>) -> Result<Arc<Expr>, String> {
    let evaled = eval_args(args, env)?;
    evaled
        .iter()
        .try_fold(0, |acc, arg| match arg.as_ref() {
            Expr::Integer { value, .. } => Ok(acc + value),
            Expr::Double { value, .. } => Ok(acc + *value as i64),
            _ => Err("add requires integer or double arguments".to_string()),
        })
        .map(|r| Arc::new(Expr::integer(r)))
}

#[lisp_fn("-")]
fn prim_sub(args: &[Arc<Expr>], env: Arc<Mutex<Env>>) -> Result<Arc<Expr>, String> {
    let evaled = eval_args(args, env)?;
    let head = evaled
        .first()
        .ok_or("sub requires at least one argument".to_string())?;
    let tail = &evaled[1..];
    let head = match head.as_ref() {
        Expr::Integer { value, .. } => *value,
        Expr::Double { value, .. } => *value as i64,
        _ => return Err("sub requires integer or double arguments".to_string()),
    };
    tail.iter()
        .try_fold(head, |acc, arg| match arg.as_ref() {
            Expr::Integer { value, .. } => Ok(acc - value),
            Expr::Double { value, .. } => Ok(acc - *value as i64),
            _ => Err("sub requires integer or double arguments".to_string()),
        })
        .map(|r| Arc::new(Expr::integer(r)))
}

#[lisp_fn("<")]
fn prim_lessthan(args: &[Arc<Expr>], env: Arc<Mutex<Env>>) -> Result<Arc<Expr>, String> {
    if args.len() != 2 {
        return Err("lessthan requires two arguments".to_string());
    }
    let evaled = eval_args(args, env)?;
    match (evaled[0].as_ref(), evaled[1].as_ref()) {
        (Expr::Integer { value: a, .. }, Expr::Integer { value: b, .. }) => {
            Ok(Arc::new(Expr::symbol(if a < b { "#t" } else { "#f" })))
        }
        _ => Err("lessthan requires integer arguments".to_string()),
    }
}

#[lisp_fn(">")]
fn prim_morethan(args: &[Arc<Expr>], env: Arc<Mutex<Env>>) -> Result<Arc<Expr>, String> {
    if args.len() != 2 {
        return Err("morethan requires two arguments".to_string());
    }
    let evaled = eval_args(args, env)?;
    match (evaled[0].as_ref(), evaled[1].as_ref()) {
        (Expr::Integer { value: a, .. }, Expr::Integer { value: b, .. }) => {
            Ok(Arc::new(Expr::symbol(if a > b { "#t" } else { "#f" })))
        }
        _ => Err("morethan requires integer arguments".to_string()),
    }
}

#[lisp_fn("<=")]
fn prim_lessthanoreq(args: &[Arc<Expr>], env: Arc<Mutex<Env>>) -> Result<Arc<Expr>, String> {
    if args.len() != 2 {
        return Err("lessthanoreq requires two arguments".to_string());
    }
    let evaled = eval_args(args, env)?;
    match (evaled[0].as_ref(), evaled[1].as_ref()) {
        (Expr::Integer { value: a, .. }, Expr::Integer { value: b, .. }) => {
            Ok(Arc::new(Expr::symbol(if a <= b { "#t" } else { "#f" })))
        }
        _ => Err("lessthanoreq requires integer arguments".to_string()),
    }
}

#[lisp_fn(">=")]
fn prim_morethanoreq(args: &[Arc<Expr>], env: Arc<Mutex<Env>>) -> Result<Arc<Expr>, String> {
    if args.len() != 2 {
        return Err("morethanoreq requires two arguments".to_string());
    }
    let evaled = eval_args(args, env)?;
    match (evaled[0].as_ref(), evaled[1].as_ref()) {
        (Expr::Integer { value: a, .. }, Expr::Integer { value: b, .. }) => {
            Ok(Arc::new(Expr::symbol(if a >= b { "#t" } else { "#f" })))
        }
        _ => Err("morethanoreq requires integer arguments".to_string()),
    }
}

pub fn assert_arg_count(args: &[Arc<Expr>], count: usize) -> Result<(), String> {
    if args.len() != count {
        return Err(format!("expected {} arguments, got {}", count, args.len()));
    }
    Ok(())
}
pub fn eval_args(args: &[Arc<Expr>], env: Arc<Mutex<Env>>) -> Result<Vec<Arc<Expr>>, String> {
    args.iter()
        .map(|arg| eval(arg.clone(), env.clone()))
        .collect()
}

#[lisp_fn("if")]
fn prim_if(args: &[Arc<Expr>], env: Arc<Mutex<Env>>) -> Result<Arc<Expr>, String> {
    if args.len() != 3 {
        return Err("if requires three arguments".to_string());
    }

    match eval(args[0].clone(), env.clone())?.as_ref() {
        Expr::Symbol { name, .. } => {
            if *name != "#f" {
                eval(args[1].clone(), env)
            } else {
                eval(args[2].clone(), env)
            }
        }
        _ => Err("First argument of if must be an integer".to_string()),
    }
}

#[lisp_fn("list")]
fn prim_list(args: &[Arc<Expr>], env: Arc<Mutex<Env>>) -> Result<Arc<Expr>, String> {
    let evaled = eval_args(args, env)?;
    Ok(Arc::new(Expr::List {
        elements: evaled,
        location: None,
        trailing_newline: false,
    }))
}

#[cfg(test)]
mod tests {

    fn default_env() -> std::sync::Arc<std::sync::Mutex<crate::lisp::env::Env>> {
        std::sync::Arc::new(std::sync::Mutex::new(crate::lisp::eval::default_env()))
    }

    use crate::lisp::parser::Value;

    use super::*;

    #[test]
    fn test_math() {
        let env = default_env();
        let expr = parser::parse_expr("(+ 1 2 (+ 1 3))").unwrap();
        assert_eq!(eval(Arc::new(expr), env), Ok(Arc::new(Expr::integer(7))));
    }

    #[test]
    fn test_lambda() {
        let env = default_env();
        let expr = parser::parse_expr("((lambda (a b) (+ a (- b 0))) 1 2)").unwrap();
        let result = eval(Arc::new(expr), env.clone());
        assert_eq!(result, Ok(Arc::new(Expr::integer(3))));
    }

    #[test]
    fn test_define() {
        let env = default_env();
        let exprs = parser::parse_file("(define a 1) a").unwrap();
        let result = eval_exprs(exprs, env.clone());
        assert_eq!(result.map(|r| r.value.clone()), Ok(Value::Integer(1)));
    }

    #[test]
    fn test_define_lambda1() {
        let env = default_env();
        let exprs = parser::parse_file("(define add (lambda(a b) (+ a b))) (add 1 2)").unwrap();
        let result = eval_exprs(exprs, env.clone());
        assert_eq!(result.map(|r| r.value.clone()), Ok(Value::Integer(3)));
    }

    #[test]
    fn test_define_lambda2() {
        let env = default_env();
        let exprs = parser::parse_file("(define (add a b) (+ a b)) (add 1 2)").unwrap();
        let result = eval_exprs(exprs, env.clone());
        assert_eq!(result.map(|r| r.value.clone()), Ok(Value::Integer(3)));
    }

    #[test]
    fn test_define_lambda3() {
        let env = default_env();
        let exprs = parser::parse_file("(define (id a) a) (id 1)").unwrap();
        assert_eq!(
            eval_exprs(exprs, env.clone()).map(|r| r.value.clone()),
            Ok(Value::Integer(1))
        );
    }
    #[test]
    fn test_if() {
        let env = default_env();
        let exprs = parser::parse_file("(if (< 1 2) 2 3)").unwrap();
        let result = eval_exprs(exprs, env.clone());
        assert_eq!(result.map(|r| r.value.clone()), Ok(Value::Integer(2)));
    }
    #[test]
    fn test_if2() {
        let env = default_env();
        let exprs = parser::parse_file("(if (< 2 1) 2 3)").unwrap();
        let result = eval_exprs(exprs, env.clone());
        assert_eq!(result.map(|r| r.value.clone()), Ok(Value::Integer(3)));
    }
    #[test]
    fn test_if3() {
        let env = default_env();
        let exprs = parser::parse_file("(if (< -3 1) 2 3)").unwrap();
        assert_eq!(
            eval_exprs(exprs, env.clone()).map(|r| r.value.clone()),
            Ok(Value::Integer(2))
        );
    }

    #[test]
    fn test_rec() {
        let env = default_env();
        let exprs = parser::parse_file(
            "(define (fib n) (if (< n 2) n (+ (fib (- n 1)) (fib (- n 2))))) (fib 10)",
        )
        .unwrap();
        let result = eval_exprs(exprs, env.clone());
        assert_eq!(result.map(|r| r.value.clone()), Ok(Value::Integer(55)));
    }

    #[test]
    fn test_define_gc() {
        use truck_polymesh::{Faces, PolygonMesh};
        let env = default_env();

        // Create and insert a test mesh
        let mesh = Arc::new(PolygonMesh::new(
            truck_polymesh::StandardAttributes::default(),
            Faces::from_tri_and_quad_faces(vec![], vec![]),
        ));
        let id = env.lock().unwrap().insert_stl(mesh);

        // Define a function that uses the mesh and evaluate it
        let exprs =
            parser::parse_file(&format!("(define (use-mesh x) (list {})) (use-mesh 1)", id))
                .unwrap();

        let result = eval_exprs(exprs, env.clone());
        assert!(result.is_ok());

        // Mesh should still be reachable
        assert!(env.lock().unwrap().get_stl(id).is_some());

        // Clear all definitions
        env.lock().unwrap().vars_mut().clear();
        env.lock().unwrap().collect_garbage();

        // Mesh should now be collected
        assert!(env.lock().unwrap().get_stl(id).is_none());
    }
}
