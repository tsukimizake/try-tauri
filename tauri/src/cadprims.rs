use crate::lisp::env::Env;
use crate::lisp::env::LispPrimitive;
use crate::lisp::eval::{assert_arg_count, eval_args};
use crate::lisp::parser::Expr;
use inventory;
use lisp_macro::lisp_fn;
use std::sync::{Arc, Mutex};

#[lisp_fn]
fn load_stl(args: &[Arc<Expr>], env: Arc<Mutex<Env>>) -> Result<Arc<Expr>, String> {
    if let Err(e) = assert_arg_count(args, 1) {
        return Err(e);
    }
    let evaled = eval_args(args, env.clone())?;
    match evaled[0].as_ref() {
        Expr::String { value: path, .. } => {
            let reader = std::fs::File::open(path).map_err(|e| e.to_string())?;

            if let Ok(mesh) =
                truck_polymesh::stl::read(&reader, truck_polymesh::stl::StlType::Automatic)
            {
                let stl_obj = Arc::new(mesh);
                let stl_id = env.lock().unwrap().insert_stl(stl_obj);
                let stl = Arc::new(Expr::stl(stl_id));
                env.lock().unwrap().insert("stl".to_string(), stl.clone());
                Ok(stl)
            } else {
                Err("load_stl: failed to read file".to_string())
            }
        }
        _ => Err("load_stl: expected string".to_string()),
    }
}

#[lisp_fn]
fn preview(args: &[Arc<Expr>], env: Arc<Mutex<Env>>) -> Result<Arc<Expr>, String> {
    if let Err(e) = assert_arg_count(args, 1) {
        return Err(e);
    }
    let evaled = eval_args(args, env.clone())?;
    match evaled[0].as_ref() {
        Expr::Stl { id, .. } => {
            env.lock().unwrap().insert_preview_list(*id);
            Ok(evaled[0].clone())
        }
        _ => Err("preview: expected stl".to_string()),
    }
}

// #[lisp_fn]
fn polygon(_args: &[Arc<Expr>], _env: Arc<Mutex<Env>>) {
    todo!()
}
