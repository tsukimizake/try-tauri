use crate::lisp::env::Env;
use crate::lisp::env::LispPrimitive;
use crate::lisp::eval::assert_arg_count;
use crate::lisp::parser::Expr;
use inventory;
use lisp_macro::lisp_fn;
use std::sync::{Arc, Mutex};
use truck_meshalgo::prelude::*;

fn add_stl_to_env(mesh: PolygonMesh, env: &Arc<Mutex<Env>>) -> Arc<Expr> {
    let stl_obj = Arc::new(mesh);
    let stl_id = env.lock().unwrap().insert_model(stl_obj);
    Arc::new(Expr::model(stl_id))
}

#[lisp_fn]
fn load_stl(args: &[Arc<Expr>], env: Arc<Mutex<Env>>) -> Result<Arc<Expr>, String> {
    if let Err(e) = assert_arg_count(args, 1) {
        return Err(e);
    }
    match args[0].as_ref() {
        Expr::String { value: path, .. } => {
            let reader = std::fs::File::open(path).map_err(|e| e.to_string())?;

            if let Ok(mesh) =
                truck_polymesh::stl::read(&reader, truck_polymesh::stl::StlType::Automatic)
            {
                let stl = add_stl_to_env(mesh, &env);
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
    match args[0].as_ref() {
        Expr::Model { id, .. } => {
            env.lock().unwrap().insert_preview_list(*id);
            Ok(args[0].clone())
        }
        _ => Err("preview: expected stl".to_string()),
    }
}

// TODO 引数evalで壊れてるの直す
#[lisp_fn]
fn triangle(args: &[Arc<Expr>], env: Arc<Mutex<Env>>) -> Result<Arc<Expr>, String> {
    if let Err(e) = assert_arg_count(args, 3) {
        return Err(e);
    }

    let positions = args
        .iter()
        .map(|expr| match expr.as_ref() {
            Expr::List { elements, .. } if elements.len() == 3 => {
                let coords: Result<Vec<f64>, String> = elements
                    .iter()
                    .map(|e| match e.as_ref() {
                        Expr::Integer { value, .. } => Ok(*value as f64),
                        Expr::Double { value, .. } => Ok(*value),
                        _ => Err("Expected number for coordinate".to_string()),
                    })
                    .collect();

                match coords {
                    Ok(c) => Ok(Point3::new(c[0], c[1], c[2])),
                    Err(e) => Err(e),
                }
            }
            _ => Err("Expected list of 3 numbers for point".to_string()),
        })
        .collect::<Result<Vec<_>, String>>()?;

    if positions.len() != 3 {
        return Err("triangle: expected exactly 3 points".to_string());
    }
    let attrs = StandardAttributes {
        positions,
        ..Default::default()
    };
    let faces = Faces::from_iter([[0, 1, 2]]);
    let polygon = PolygonMesh::new(attrs, faces);

    Ok(add_stl_to_env(polygon, &env))
}
