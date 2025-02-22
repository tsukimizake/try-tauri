use crate::lisp::eval::{assert_arg_count, eval_args};
use crate::lisp::Expr;
use std::sync::{Arc, Mutex};

pub fn prim_load_stl(args: &[Arc<Expr>], env: Arc<Mutex<crate::lisp::env::Env>>) -> Result<Arc<Expr>, String> {
    if let Err(e) = assert_arg_count(args, 1) {
        return Err(e);
    }
    let evaled = eval_args(args, env.clone())?;
    match evaled[0].as_ref() {
        Expr::String { value: path, .. } => {
            // std::io::Read
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

pub fn prim_preview(args: &[Arc<Expr>], env: Arc<Mutex<crate::lisp::env::Env>>) -> Result<Arc<Expr>, String> {
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
