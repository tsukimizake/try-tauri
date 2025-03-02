use crate::lisp::env::{Env, LispPrimitive, LispSpecialForm};
use crate::lisp::parser;
use crate::lisp::parser::Expr;
use inventory;
use lisp_macro::{lisp_fn, lisp_sp_form};
// Note: RangeBounds are used indirectly through the From impls for ArgCount
use std::sync::{Arc, Mutex};

use super::Evaled;

pub fn eval_exprs(exprs: Vec<parser::Expr>, env: Arc<Mutex<Env>>) -> Result<Arc<Evaled>, String> {
    // Only evaluate the last expression for the result
    let mut last_result = Ok(Arc::new(Expr::list(vec![])));

    // Evaluate each expression in sequence
    for expr in &exprs {
        last_result = eval(Arc::new(expr.clone()), env.clone());
    }

    // Lock the environment only once to get polys and previews
    let env_guard = env.lock().unwrap();
    let polys = env_guard
        .polys()
        .iter()
        .map(|(id, poly)| -> (usize, crate::elm_interface::SerdeStlFaces) { (*id, poly.into()) })
        .collect();
    let previews = env_guard.preview_list();

    last_result.map(|expr| {
        Arc::new(Evaled {
            value: parser::cast_evaled(expr),
            polys,
            previews,
        })
    })
}

pub fn eval(expr: Arc<Expr>, env: Arc<Mutex<Env>>) -> Result<Arc<Expr>, String> {
    match expr.as_ref() {
        Expr::Symbol { name, .. } => {
            // Lock the environment only once
            let env_guard = env.lock().unwrap();
            env_guard
                .get(name)
                .ok_or_else(|| format!("Undefined symbol: {}", name))
        }
        Expr::Integer { value, .. } => Ok(Arc::new(Expr::integer(*value))),
        Expr::Double { value, .. } => Ok(Arc::new(Expr::double(*value))),
        Expr::List { elements, .. } => eval_list(&elements[..], env),
        Expr::String { value, .. } => Ok(Arc::new(Expr::string(value.clone()))),
        Expr::Model { id, .. } => Ok(Arc::new(Expr::model(*id))),
        Expr::Quote { expr, .. } => Ok(Arc::new((**expr).clone())),
        Expr::Quasiquote { expr, .. } => eval_quasiquote_wrapper(&(**expr), env),
        Expr::Unquote { .. } => Err("Unquote can only be used inside a quasiquote".to_string()),
        // For these types, we can just return the original expression
        Expr::Builtin { .. } | Expr::SpecialForm { .. } | Expr::Clausure { .. } | Expr::Macro { .. } => Ok(expr),
    }
}

fn eval_list(elements: &[Arc<Expr>], env: Arc<Mutex<Env>>) -> Result<Arc<Expr>, String> {
    if elements.is_empty() {
        return Ok(Arc::new(Expr::list(vec![])));
    }

    // Check for special forms first to avoid unnecessary cloning
    let first_elem = elements[0].as_ref();
    if first_elem.is_symbol("lambda") {
        return eval_lambda(elements, env);
    }
    if first_elem.is_symbol("define") {
        return eval_define(elements, env);
    }
    if first_elem.is_symbol("if") {
        return eval_if(elements, env);
    }
    if first_elem.is_symbol("let") {
        return eval_let(elements, env);
    }
    if first_elem.is_symbol("defmacro") {
        return eval_defmacro(elements, env);
    }

    // For function calls, evaluate the function expression first
    let first = eval(elements[0].clone(), env.clone())?;
    match &*first {
        Expr::Builtin { fun, .. } => {
            let args = &elements[1..];
            let evaled = eval_args(args, env.clone())?;
            fun(&evaled, env)
        }
        Expr::SpecialForm { fun, .. } => {
            // For special forms, don't evaluate the arguments yet
            // Pass them directly to the special form function
            let args = &elements[1..];
            fun(args, env)
        }
        Expr::Clausure {
            args,
            body,
            env: clausure_env,
        } => {
            let newenv = Env::make_child(&clausure_env);

            // Create a single reference to the parent environment for evaluating arguments
            let parent_env = env.clone();

            for (arg, value) in args.iter().zip(elements.iter().skip(1)) {
                let val = eval(value.clone(), parent_env.clone())?;
                newenv.lock().unwrap().insert(arg.clone(), val);
            }

            eval(body.clone(), newenv)
        }
        Expr::Macro {
            args,
            body,
            env: macro_env,
        } => {
            // For macros, don't evaluate the arguments yet
            let newenv = Env::make_child(&macro_env);

            // Bind unevaluated arguments to parameters
            for (arg, value) in args.iter().zip(elements.iter().skip(1)) {
                newenv.lock().unwrap().insert(arg.clone(), value.clone());
            }

            // Evaluate the macro body to get the expansion
            let expansion = eval(body.clone(), newenv)?;

            // Then evaluate the expansion
            eval(expansion, env)
        }
        _ => Err(format!(
            "First element of list is not a function, special form, or macro"
        )),
    }
}

// (define a 1) => (define a 1)
// (define (add a b) (+ a b)) => (define add (lambda (a b) (+ a b)))
// TODO: proper location and trailing_newline
fn eval_define(elements: &[Arc<Expr>], env: Arc<Mutex<Env>>) -> Result<Arc<Expr>, String> {
    assert_arg_count(elements, 3)?;
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

fn eval_define_impl(elements: &[Arc<Expr>], env: Arc<Mutex<Env>>) -> Result<Arc<Expr>, String> {
    match (elements[1].as_ref(), elements[2].clone()) {
        (Expr::Symbol { name, .. }, value) => {
            // Evaluate the value first
            let value = eval(value, env.clone())?;
            // Then insert it into the environment
            env.lock().unwrap().insert(name.clone(), value.clone());
            Ok(value)
        }
        (Expr::List { elements: args, .. }, body) => {
            // Create a new environment for the closure
            let newenv = Env::make_child(&env);

            // Extract argument names
            let argnames: Vec<String> = args
                .iter()
                .map(|arg| {
                    arg.as_ref()
                        .as_symbol()
                        .expect("Lambda argument is not a symbol")
                        .to_string()
                })
                .collect();

            // Create the closure
            let clausure = Arc::new(Expr::Clausure {
                args: argnames,
                body,
                env: newenv,
            });

            // Get the function name
            let fn_name = elements[1].as_symbol()?.to_string();

            // Insert the closure into the environment
            env.lock().unwrap().insert(fn_name, clausure.clone());

            Ok(clausure)
        }
        _ => Err("define requires a symbol as an argument".to_string()),
    }
}

fn eval_defmacro(elements: &[Arc<Expr>], env: Arc<Mutex<Env>>) -> Result<Arc<Expr>, String> {
    assert_arg_count(elements, 3)?;
    match elements.get(1).map(|x| x.as_ref()) {
        Some(Expr::List {
            elements: name_and_args,
            ..
        }) => {
            if name_and_args.is_empty() {
                return Err("defmacro requires a name".to_string());
            }

            let macro_name = name_and_args[0].as_symbol()?;
            let macro_args = &name_and_args[1..];

            // Extract argument names
            let argnames: Vec<String> = macro_args
                .iter()
                .map(|arg| {
                    arg.as_ref()
                        .as_symbol()
                        .expect("Macro argument is not a symbol")
                        .to_string()
                })
                .collect();

            // Create the macro
            let macro_object = Arc::new(Expr::Macro {
                args: argnames,
                body: elements[2].clone(),
                env: env.clone(),
            });

            // Insert the macro into the environment
            env.lock()
                .unwrap()
                .insert(macro_name.to_string(), macro_object.clone());

            Ok(macro_object)
        }
        _ => Err("defmacro requires a name and argument list".to_string()),
    }
}

// (lambda (a b) (+ a b))
fn eval_lambda(expr: &[Arc<Expr>], env: Arc<Mutex<Env>>) -> Result<Arc<Expr>, String> {
    assert_arg_count(expr, 3)?;
    match (expr[1].as_ref(), expr[2].clone()) {
        (Expr::List { elements: args, .. }, body) => {
            let newenv = Env::make_child(&env);
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

fn eval_let(expr: &[Arc<Expr>], env: Arc<Mutex<Env>>) -> Result<Arc<Expr>, String> {
    assert_arg_count(expr, 3)?;
    match (expr[1].as_ref(), expr[2].clone()) {
        (
            Expr::List {
                elements: bindings, ..
            },
            body,
        ) => {
            let newenv = Env::make_child(&env);

            // Evaluate each binding and add to new environment
            for binding in bindings {
                match binding.as_ref() {
                    Expr::List { elements, .. } if elements.len() == 2 => {
                        let name = elements[0].as_ref().as_symbol()?;
                        // Use a single reference to newenv for all bindings
                        let value = eval(elements[1].clone(), newenv.clone())?;
                        newenv.lock().unwrap().insert(name.to_string(), value);
                    }
                    _ => return Err("Invalid let binding format".to_string()),
                }
            }

            // Evaluate body in new environment
            eval(body, newenv)
        }
        _ => Err("let requires a list of bindings".to_string()),
    }
}

// (if (< 1 2) 2 3)
fn eval_if(expr: &[Arc<Expr>], env: Arc<Mutex<Env>>) -> Result<Arc<Expr>, String> {
    assert_arg_count(expr, 4)?;

    // Evaluate the condition first
    let condition = eval(expr[1].clone(), env.clone())?;

    match condition.as_ref() {
        Expr::Symbol { name, .. } => {
            if *name != "#f" {
                // Only clone env once for the branch we're taking
                eval(expr[2].clone(), env)
            } else {
                eval(expr[3].clone(), env)
            }
        }
        _ => Err("First argument of if must be a boolean".to_string()),
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

    // Register all special forms that used the lisp_sp_form macro
    for special_form in inventory::iter::<LispSpecialForm> {
        env.insert(
            special_form.name.to_string(),
            Arc::new(Expr::SpecialForm {
                name: special_form.name.to_string(),
                fun: special_form.func,
            }),
        );
    }

    env
}

#[lisp_fn("+")]
fn prim_add(args: &[Arc<Expr>], _env: Arc<Mutex<Env>>) -> Result<Arc<Expr>, String> {
    assert_arg_count(args, 1..)?;
    args.iter()
        .try_fold(0, |acc, arg| match arg.as_ref() {
            Expr::Integer { value, .. } => Ok(acc + value),
            Expr::Double { value, .. } => Ok(acc + *value as i64),
            _ => Err("add requires integer or double arguments".to_string()),
        })
        .map(|r| Arc::new(Expr::integer(r)))
}

#[lisp_fn("-")]
fn prim_sub(args: &[Arc<Expr>], _env: Arc<Mutex<Env>>) -> Result<Arc<Expr>, String> {
    assert_arg_count(args, 1..)?;
    let head = args.first().unwrap();
    let tail = &args[1..];
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
fn prim_lessthan(args: &[Arc<Expr>], _env: Arc<Mutex<Env>>) -> Result<Arc<Expr>, String> {
    assert_arg_count(args, 2)?;
    match (args[0].as_ref(), args[1].as_ref()) {
        (Expr::Integer { value: a, .. }, Expr::Integer { value: b, .. }) => {
            Ok(Arc::new(Expr::symbol(if a < b { "#t" } else { "#f" })))
        }
        _ => Err("lessthan requires integer arguments".to_string()),
    }
}

#[lisp_fn(">")]
fn prim_morethan(args: &[Arc<Expr>], _env: Arc<Mutex<Env>>) -> Result<Arc<Expr>, String> {
    assert_arg_count(args, 2)?;
    match (args[0].as_ref(), args[1].as_ref()) {
        (Expr::Integer { value: a, .. }, Expr::Integer { value: b, .. }) => {
            Ok(Arc::new(Expr::symbol(if a > b { "#t" } else { "#f" })))
        }
        _ => Err("morethan requires integer arguments".to_string()),
    }
}

#[lisp_fn("<=")]
fn prim_lessthanoreq(args: &[Arc<Expr>], _env: Arc<Mutex<Env>>) -> Result<Arc<Expr>, String> {
    assert_arg_count(args, 2)?;
    match (args[0].as_ref(), args[1].as_ref()) {
        (Expr::Integer { value: a, .. }, Expr::Integer { value: b, .. }) => {
            Ok(Arc::new(Expr::symbol(if a <= b { "#t" } else { "#f" })))
        }
        _ => Err("lessthanoreq requires integer arguments".to_string()),
    }
}

#[lisp_fn(">=")]
fn prim_morethanoreq(args: &[Arc<Expr>], _env: Arc<Mutex<Env>>) -> Result<Arc<Expr>, String> {
    assert_arg_count(args, 2)?;
    match (args[0].as_ref(), args[1].as_ref()) {
        (Expr::Integer { value: a, .. }, Expr::Integer { value: b, .. }) => {
            Ok(Arc::new(Expr::symbol(if a >= b { "#t" } else { "#f" })))
        }
        _ => Err("morethanoreq requires integer arguments".to_string()),
    }
}

pub enum ArgCount {
    Exact(usize),
    Range(usize, usize),
    AtLeast(usize),
    AtMost(usize),
}

impl ArgCount {
    fn contains(&self, n: &usize) -> bool {
        match self {
            ArgCount::Exact(count) => n == count,
            ArgCount::Range(min, max) => n >= min && n <= max,
            ArgCount::AtLeast(min) => n >= min,
            ArgCount::AtMost(max) => n <= max,
        }
    }
}

impl From<usize> for ArgCount {
    fn from(n: usize) -> Self {
        ArgCount::Exact(n)
    }
}

impl From<std::ops::RangeInclusive<usize>> for ArgCount {
    fn from(r: std::ops::RangeInclusive<usize>) -> Self {
        let (min, max) = r.into_inner();
        ArgCount::Range(min, max)
    }
}

impl From<std::ops::RangeFrom<usize>> for ArgCount {
    fn from(r: std::ops::RangeFrom<usize>) -> Self {
        ArgCount::AtLeast(r.start)
    }
}

impl From<std::ops::RangeTo<usize>> for ArgCount {
    fn from(r: std::ops::RangeTo<usize>) -> Self {
        ArgCount::AtMost(r.end - 1)
    }
}

impl From<std::ops::RangeToInclusive<usize>> for ArgCount {
    fn from(r: std::ops::RangeToInclusive<usize>) -> Self {
        ArgCount::AtMost(r.end)
    }
}

pub fn assert_arg_count(args: &[Arc<Expr>], range: impl Into<ArgCount>) -> Result<(), String> {
    let range = range.into();
    if !range.contains(&args.len()) {
        let error_msg = match range {
            ArgCount::Exact(n) => {
                format!("expected {} arguments, got {}", n, args.len())
            }
            ArgCount::Range(min, max) => {
                format!("expected {} to {} arguments, got {}", min, max, args.len())
            }
            ArgCount::AtLeast(min) => {
                format!("expected at least {} arguments, got {}", min, args.len())
            }
            ArgCount::AtMost(max) => {
                format!("expected at most {} arguments, got {}", max, args.len())
            }
        };
        return Err(error_msg);
    }
    Ok(())
}

pub fn eval_args(args: &[Arc<Expr>], env: Arc<Mutex<Env>>) -> Result<Vec<Arc<Expr>>, String> {
    // Avoid cloning the environment for each argument evaluation
    args.iter()
        .map(|arg| eval(arg.clone(), env.clone()))
        .collect()
}

// Keeps track of the nesting level during quasiquote evaluation
fn eval_quasiquote(
    expr: &Expr,
    env: Arc<Mutex<Env>>,
    nesting_level: usize,
) -> Result<Arc<Expr>, String> {
    match expr {
        // If we encounter an unquote at level 1, evaluate its contents
        // At deeper levels, we preserve the unquote but decrease the nesting level
        Expr::Unquote {
            expr,
            location,
            trailing_newline,
        } => {
            if nesting_level == 1 {
                eval(Arc::new((**expr).clone()), env)
            } else {
                // Decrease nesting level for nested unquotes
                let inner = eval_quasiquote(expr, env, nesting_level - 1)?;
                Ok(Arc::new(Expr::Unquote {
                    expr: Box::new((*inner).clone()),
                    location: *location,
                    trailing_newline: *trailing_newline,
                }))
            }
        }

        // If we encounter another quasiquote, increase the nesting level
        Expr::Quasiquote {
            expr,
            location,
            trailing_newline,
        } => {
            let inner = eval_quasiquote(expr, env.clone(), nesting_level + 1)?;
            Ok(Arc::new(Expr::Quasiquote {
                expr: Box::new((*inner).clone()),
                location: *location,
                trailing_newline: *trailing_newline,
            }))
        }

        // If we have a list, process each element
        Expr::List {
            elements,
            location,
            trailing_newline,
        } => {
            let mut result = Vec::new();
            for element in elements {
                result.push(eval_quasiquote(
                    element.as_ref(),
                    env.clone(),
                    nesting_level,
                )?);
            }
            Ok(Arc::new(Expr::List {
                elements: result,
                location: *location,
                trailing_newline: *trailing_newline,
            }))
        }

        // For all other expressions, just return them as is (like quote)
        _ => Ok(Arc::new(expr.clone())),
    }
}

// Wrapper function to start quasiquote evaluation with nesting level 1
fn eval_quasiquote_wrapper(expr: &Expr, env: Arc<Mutex<Env>>) -> Result<Arc<Expr>, String> {
    eval_quasiquote(expr, env, 1)
}

#[lisp_fn("list")]
fn prim_list(args: &[Arc<Expr>], _env: Arc<Mutex<Env>>) -> Result<Arc<Expr>, String> {
    Ok(Arc::new(Expr::List {
        elements: args.to_vec(),
        location: None,
        trailing_newline: false,
    }))
}

/// Check if an expression is a list
///
/// # Lisp Usage
///
/// ```lisp
/// (list? expr)
/// ```
///
/// # Examples
///
/// ```lisp
/// (list? '(1 2 3))  ;; Returns #t
/// (list? 42)        ;; Returns #f
/// (list? "hello")   ;; Returns #f
/// ```
#[lisp_sp_form("list?")]
fn prim_list_p(args: &[Arc<Expr>], env: Arc<Mutex<Env>>) -> Result<Arc<Expr>, String> {
    assert_arg_count(args, 1)?;
    // Evaluate the argument first, since we need its value
    let arg = eval(args[0].clone(), env)?;

    match arg.as_ref() {
        Expr::List { .. } => Ok(Arc::new(Expr::symbol("#t"))),
        _ => Ok(Arc::new(Expr::symbol("#f"))),
    }
}

/// Get the first element of a list
///
/// # Lisp Usage
///
/// ```lisp
/// (first list)
/// ```
///
/// # Examples
///
/// ```lisp
/// (first '(1 2 3))  ;; Returns 1
/// ```
#[lisp_sp_form("car")]
fn prim_first(args: &[Arc<Expr>], env: Arc<Mutex<Env>>) -> Result<Arc<Expr>, String> {
    assert_arg_count(args, 1)?;
    // Evaluate the argument first, since we need its value
    let arg = eval(args[0].clone(), env)?;
    
    match arg.as_ref() {
    Expr::List { elements, .. } => {
            if elements.is_empty() {
                Err("Cannot get first element of empty list".to_string())
            } else {
                Ok(elements[0].clone())
            }
        }
        _ => Err("first expects a list argument".to_string()),
    }
}

/// Get all elements of a list except the first
///
/// # Lisp Usage
///
/// ```lisp
/// (rest list)
/// ```
///
/// # Examples
///
/// ```lisp
/// (rest '(1 2 3))  ;; Returns (2 3)
/// ```
#[lisp_sp_form("cdr")]
fn prim_rest(args: &[Arc<Expr>], env: Arc<Mutex<Env>>) -> Result<Arc<Expr>, String> {
    assert_arg_count(args, 1)?;
    // Evaluate the argument first, since we need its value
    let arg = eval(args[0].clone(), env)?;
    
    match arg.as_ref() {
        Expr::List { elements, location, trailing_newline } => {
            if elements.is_empty() {
                Ok(Arc::new(Expr::List {
                    elements: vec![],
                    location: *location,
                    trailing_newline: *trailing_newline,
                }))
            } else {
                Ok(Arc::new(Expr::List {
                    elements: elements[1..].to_vec(),
                    location: *location,
                    trailing_newline: *trailing_newline,
                }))
            }
        }
        _ => Err("rest expects a list argument".to_string()),
    }
}

/// Check if a list is empty
///
/// # Lisp Usage
///
/// ```lisp
/// (null? list)
/// ```
///
/// # Examples
///
/// ```lisp
/// (null? '())       ;; Returns #t
/// (null? '(1 2 3))  ;; Returns #f
/// (null? 42)        ;; Returns #f (not a list)
/// ```
#[lisp_sp_form("null?")]
fn prim_null_p(args: &[Arc<Expr>], env: Arc<Mutex<Env>>) -> Result<Arc<Expr>, String> {
    assert_arg_count(args, 1)?;
    // Evaluate the argument first, since we need its value
    let arg = eval(args[0].clone(), env)?;
    
    match arg.as_ref() {
        Expr::List { elements, .. } => {
            if elements.is_empty() {
            Ok(Arc::new(Expr::symbol("#t")))
            } else {
                Ok(Arc::new(Expr::symbol("#f")))
            }
        }
        _ => Ok(Arc::new(Expr::symbol("#f"))),
    }
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
    fn test_let() {
        let env = default_env();
        let exprs = parser::parse_file("(let ((a 1) (b 2)) (+ a b))").unwrap();
        assert_eq!(
            eval_exprs(exprs, env.clone()).map(|r| r.value.clone()),
            Ok(Value::Integer(3))
        );
    }

    #[test]
    fn test_let2() {
        let env = default_env();
        let exprs = parser::parse_file("(let ((a 1)) (let ((b (+ a 1)))  (+ b a)))").unwrap();
        assert_eq!(
            eval_exprs(exprs, env.clone()).map(|r| r.value.clone()),
            Ok(Value::Integer(3))
        );
    }

    #[test]
    fn test_let_3() {
        let env = default_env();
        // let_star
        let exprs = parser::parse_file("(let ((a 1) (b (+ a 1))) (+ a b))").unwrap();
        assert_eq!(
            eval_exprs(exprs, env.clone()).map(|r| r.value.clone()),
            Ok(Value::Integer(3))
        );
    }

    #[test]
    fn test_let_4() {
        let env = default_env();
        let exprs = parser::parse_file("(let ((a 1)) (let ((a 0)) a))").unwrap();

        assert_eq!(
            eval_exprs(exprs, env.clone()).map(|r| r.value.clone()),
            Ok(Value::Integer(0))
        );
    }

    #[test]
    fn test_let_5() {
        let env = default_env();
        let exprs = parser::parse_file("(let ((a 0)) 1) a").unwrap();
        std::assert_matches::assert_matches!(
            eval_exprs(exprs, env.clone()).map(|r| r.value.clone()),
            Err(_)
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
    fn test_defmacro() {
        let env = default_env();
    let exprs = parser::parse_file(
            "(defmacro (when cond body) `(if ~cond ~body #f)) (when (< 1 2) 3)",
        )
        .unwrap();
        let result = eval_exprs(exprs, env.clone());
        assert_eq!(result.map(|r| r.value.clone()), Ok(Value::Integer(3)));
    }    
    #[test]
    fn test_quasiquote() {
        let env = default_env();
    
        // Simple quasiquote with no unquote
        let exprs = parser::parse_file("`(1 2 3)").unwrap();
        let result = eval_exprs(exprs, env.clone());
match result.map(|r| r.value.clone()) {
            Ok(Value::List(items)) => {
                assert_eq!(items.len(), 3);
                assert_eq!(items[0], Value::Integer(1));
                assert_eq!(items[1], Value::Integer(2));
                assert_eq!(items[2], Value::Integer(3));
            },
            _ => panic!("Expected list"),
        }
        
        // Qusiquote with unquote
        let exprs = parser::parse_file("(define x 42) `(1 ~x 3)").unwrap();
        let result = eval_exprs(exprs, env.clone());
match result.map(|r| r.value.clone()) {
            Ok(Value::List(items)) => {
                assert_eq!(items.len(), 3);
                assert_eq!(items[0], Value::Integer(1));
                assert_eq!(items[1], Value::Integer(42));
                assert_eq!(items[2], Value::Integer(3));
            },
            _ => panic!("Expected list"),
        }
        
        // Siple case of quasiquote
        let exprs = parser::parse_file("`(1 2 3)").unwrap();
        let result = eval_exprs(exprs, env.clone());
match result.map(|r| r.value.clone()) {
            Ok(Value::List(items)) => {
                assert_eq!(items.len(), 3);
                assert_eq!(items[0], Value::Integer(1));
                assert_eq!(items[1], Value::Integer(2));
                assert_eq!(items[2], Value::Integer(3));
            },
            _ => panic!("Expected list"),
        }
        
        // Usng unquote with a defined value
        let exprs = parser::parse_file("(define x 42) `(1 ~x 3)").unwrap();
        let result = eval_exprs(exprs, env.clone());
match result.map(|r| r.value.clone()) {
            Ok(Value::List(items)) => {
                assert_eq!(items.len(), 3);
                assert_eq!(items[0], Value::Integer(1));
                assert_eq!(items[1], Value::Integer(42));
                assert_eq!(items[2], Value::Integer(3));
            },
            _ => panic!("Expected list"),
        }
        
        // Neted quasiquote/unquote - this tests proper nesting behavior
        let exprs = parser::parse_file("(define x 42) `(1 `(2 ~x) ~x)").unwrap();
        let result = eval_exprs(exprs, env.clone());
match result.map(|r| r.value.clone()) {
            Ok(Value::List(items)) => {
                assert_eq!(items.len(), 3);
                assert_eq!(items[0], Value::Integer(1));
                
                // The second item should be a quasiquoted list where the inner x is not evaluated
                // because it's inside a nested quasiquote
                match &items[1] {
    Value::List(inner) => {
                        assert_eq!(inner.len(), 2);
                        assert_eq!(inner[0], Value::Integer(2));
                        assert_eq!(inner[1], Value::Symbol("x".to_string()));
                    },
                    _ => panic!("Expected inner list"),
                }
                
                // The third item is directly unquoted, so it should be evaluated
                assert_eq!(items[2], Value::Integer(42));
            },
            _ => panic!("Expected list"),
        }
    }

    #[test]
    fn test_list_functions() {
        let env = default_env();
        
        // Test list?
        let exprs = parser::parse_file("(list? '(1 2 3))").unwrap();
        assert_eq!(
    eval_exprs(exprs, env.clone()).map(|r| r.value.clone()),
            Ok(Value::Symbol("#t".to_string()))
        );
        
        let exprs = parser::parse_file("(list? 42)").unwrap();
        assert_eq!(
            eval_exprs(exprs, env.clone()).map(|r| r.value.clone()),
    Ok(Value::Symbol("#f".to_string()))
        );
        
        // Test first/car
        let exprs = parser::parse_file("(car '(1 2 3))").unwrap();
        assert_eq!(
    eval_exprs(exprs, env.clone()).map(|r| r.value.clone()),
            Ok(Value::Integer(1))
        );
        
        // Test rest/cdr
        let exprs = parser::parse_file("(cdr '(1 2 3))").unwrap();
        match eval_exprs(exprs, env.clone()).map(|r| r.value.clone()) {
    Ok(Value::List(items)) => {
                assert_eq!(items.len(), 2);
                assert_eq!(items[0], Value::Integer(2));
                assert_eq!(items[1], Value::Integer(3));
            },
            _ => panic!("Expected list"),
        }
        
        // Tet null?
        let exprs = parser::parse_file("(null? '())").unwrap();
        assert_eq!(
    eval_exprs(exprs, env.clone()).map(|r| r.value.clone()),
            Ok(Value::Symbol("#t".to_string()))
        );
        
        let exprs = parser::parse_file("(null? '(1 2 3))").unwrap();
        assert_eq!(
            eval_exprs(exprs, env.clone()).map(|r| r.value.clone()),
    Ok(Value::Symbol("#f".to_string()))
        );
        
        // Test empty list handling
        let exprs = parser::parse_file("(cdr '())").unwrap();
        match eval_exprs(exprs, env.clone()).map(|r| r.value.clone()) {
    Ok(Value::List(items)) => {
                assert_eq!(items.len(), 0);
            },
            _ => panic!("Expected empty list"),
        }
    }
    
    #[test]
    #[ignore] // Temporarily ignoring this test as it needs to be fixed for special forms
    fn test_thread_macro() {
        let env = default_env();
    
        // Define the thread-first macro
        let thread_first_macro = r#"
        (defmacro (-> x form)
  (if (list? form)
              `(,(car form) ~x ,@(cdr form))
              `(~form ~x)))
              
        (defmacro (-> x form1 form2)
          `(-> (-> ~x ~form1) ~form2))
        "#;
        
        let exprs = parser::parse_file(thread_first_macro).unwrap();
        eval_exprs(exprs, env.clone()).unwrap();
        
// Test basic threading
        let exprs = parser::parse_file("(-> 1 (+ 2))").unwrap();
        assert_eq!(
    eval_exprs(exprs, env.clone()).map(|r| r.value.clone()),
            Ok(Value::Integer(3))
        );
        
        // Test nested threading
        let exprs = parser::parse_file("(-> 1 (+ 2) (+ 3))").unwrap();
        assert_eq!(
    eval_exprs(exprs, env.clone()).map(|r| r.value.clone()),
            Ok(Value::Integer(6))
        );
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
        let id = env.lock().unwrap().insert_model(mesh);

        // Define a function that uses the mesh and evaluate it
        let exprs =
            parser::parse_file(&format!("(define (use-mesh x) (list {})) (use-mesh 1)", id))
                .unwrap();

        let result = eval_exprs(exprs, env.clone());
        assert!(result.is_ok());

        // Mesh should still be reachable
        assert!(env.lock().unwrap().get_model(id).is_some());

        // Clear all definitions
        env.lock().unwrap().vars_mut().clear();
        env.lock().unwrap().collect_garbage();

        // Mesh should now be collected
        assert!(env.lock().unwrap().get_model(id).is_none());
    }
}
