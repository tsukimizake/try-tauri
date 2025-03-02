use crate::lisp::env::Env;
use crate::lisp::env::LispPrimitive;
use crate::lisp::env::Model;
use crate::lisp::env::extract;
use crate::lisp::eval::assert_arg_count;
use crate::lisp::parser::Expr;
use inventory;
use lisp_macro::lisp_fn;
use std::sync::{Arc, Mutex};
use truck_meshalgo::prelude::*;
use truck_modeling::{Point3, builder};
use truck_topology::EdgeDisplayFormat;
use truck_topology::VertexDisplayFormat;
use truck_topology::WireDisplayFormat;

fn return_model<T: Into<Model>>(model_into: T, env: Arc<Mutex<Env>>) -> Result<Arc<Expr>, String> {
    let model = model_into.into();
    let id = env.lock().unwrap().insert_model(model);
    Ok(Arc::new(Expr::model(id)))
}

/// Load an STL file into the environment
///
/// # Lisp Usage
///
/// ```lisp
/// (load-stl "path/to/file.stl")
/// ```
///
/// # Returns
///
/// A model expression representing the loaded STL file.
/// The model is also bound to the variable `stl` in the environment.
#[lisp_fn]
fn load_stl(args: &[Arc<Expr>], env: Arc<Mutex<Env>>) -> Result<Arc<Expr>, String> {
    assert_arg_count(args, 1)?;
    match args[0].as_ref() {
        Expr::String { value: path, .. } => {
            let reader = std::fs::File::open(path).map_err(|e| e.to_string())?;

            if let Ok(mesh) =
                truck_polymesh::stl::read(&reader, truck_polymesh::stl::StlType::Automatic)
            {
                return_model(Arc::new(mesh), env)
            } else {
                Err("load_stl: failed to read file".to_string())
            }
        }
        _ => Err("load_stl: expected string".to_string()),
    }
}

/// Mark a model for preview/rendering in the UI
///
/// # Lisp Usage
///
/// ```lisp
/// (preview model)
/// ```
///
/// # Returns
///
/// The model that was marked for preview
#[lisp_fn]
fn preview(args: &[Arc<Expr>], env: Arc<Mutex<Env>>) -> Result<Arc<Expr>, String> {
    assert_arg_count(args, 1)?;
    match args[0].as_ref() {
        Expr::Model { id, .. } => {
            // Get the model and verify it's a mesh
            let mut env_guard = env.lock().unwrap();
            let model = env_guard
                .get_model(*id)
                .ok_or_else(|| format!("Model with id {} not found", id))?;

            // Check if the model is a mesh
            if model.as_mesh().is_none() {
                return Err("preview: expected mesh model".to_string());
            }

            // Add to preview list using the same lock
            env_guard.insert_preview_list(*id);
            Ok(args[0].clone())
        }
        _ => Err("preview: expected mesh model".to_string()),
    }
}

/// Create a point at the specified coordinates
///
/// # Lisp Usage
///
/// This function is available as both `point` and `p` in Lisp:
///
/// ```lisp
/// (point x y z)  ;; Using the full name with 3 coordinates
/// (point x y)    ;; Using the full name with 2 coordinates (z=0)
/// (p x y z)      ;; Using the shorthand alias with 3 coordinates
/// (p x y)        ;; Using the shorthand alias with 2 coordinates (z=0)
/// ```
///
/// # Examples
///
/// ```lisp
/// (point 1 2 3)  ;; Create a point at (1, 2, 3)
/// (p 0 0 0)      ;; Create a point at origin using the shorthand
/// (p 10 5)       ;; Create a point at (10, 5, 0) - z defaults to 0
///
/// ;; Create multiple points
/// (define p1 (p 0 0 0))
/// (define p2 (p 1 0))    ;; z defaults to 0
/// (define p3 (p 0 1 0))
/// ```
///
/// # Returns
///
/// A model expression representing the created point
#[lisp_fn("p")]
fn point(args: &[Arc<Expr>], env: Arc<Mutex<Env>>) -> Result<Arc<Expr>, String> {
    assert_arg_count(args, 2..=3).map_err(|e| format!("point: {}", e))?;

    let mut coords = args
        .iter()
        .map(|expr| extract::number(expr.as_ref()))
        .collect::<Result<Vec<_>, String>>()?;

    if coords.len() == 2 {
        coords.push(0.0);
    }

    let point = truck_modeling::Point3::new(coords[0], coords[1], coords[2]);
    return_model(Model::Point3(point), env)
}

/// Create a line between two points
///
/// # Lisp Usage
///
/// ```lisp
/// (line point1 point2)
/// ```
///
/// # Examples
///
/// ```lisp
/// (define p1 (p 0 0 0))
/// (define p2 (p 1 1 1))
/// (line p1 p2)  ;; Create a line from origin to (1,1,1)
///
/// ;; Create a square using lines
/// (define p1 (p 0 0 0))
/// (define p2 (p 1 0 0))
/// (define p3 (p 1 1 0))
/// (define p4 (p 0 1 0))
/// (define l1 (line p1 p2))
/// (define l2 (line p2 p3))
/// (define l3 (line p3 p4))
/// (define l4 (line p4 p1))
/// ```
///
/// # Returns
///
/// A model expression representing the created line (edge)
#[lisp_fn]
fn line(args: &[Arc<Expr>], env: Arc<Mutex<Env>>) -> Result<Arc<Expr>, String> {
    assert_arg_count(args, 2)?;

    let points = args
        .iter()
        .map(|expr| extract::point3(expr.as_ref(), &env))
        .collect::<Result<Vec<_>, String>>()?;

    let v1 = Arc::new(truck_modeling::Vertex::new(points[0]));
    let v2 = Arc::new(truck_modeling::Vertex::new(points[1]));
    let edge = truck_modeling::builder::line(&v1, &v2);
    return_model(Model::Edge(Arc::new(edge)), env)
}

/// turtle_sketch to create a face from a sequence of points.
///
/// # Lisp Usage
///
/// ```lisp
/// (turtle point1 point2 point3 ...)
/// ```
///
/// # Examples
///
/// ```lisp
/// ;; Create a square face
/// (turtle (p 0 0) (p 1 0) (p 1 1) (p 0 1))
/// ```
///
/// # Returns
///
/// A model expression representing the created face
#[lisp_fn("turtle")]
fn turtle_sketch(args: &[Arc<Expr>], env: Arc<Mutex<Env>>) -> Result<Arc<Expr>, String> {
    assert_arg_count(args, 3..).map_err(|e| format!("turtle: {}", e))?;

    let points = args
        .iter()
        .map(|expr| extract::point3(expr.as_ref(), &env))
        .collect::<Result<Vec<_>, String>>()?;

    let mut edges = Vec::new();

    // Create vertices from points
    let mut vertices: Vec<Arc<truck_modeling::Vertex>> = Vec::new();
    let mut current_point = points[0];

    let first_vertex = Arc::new(truck_modeling::Vertex::new(current_point));
    vertices.push(first_vertex.clone());

    for i in 1..points.len() {
        let movement = points[i];
        let next_point = truck_modeling::Point3::new(
            current_point.x + movement.x,
            current_point.y + movement.y,
            current_point.z + movement.z,
        );

        let next_vertex = Arc::new(truck_modeling::Vertex::new(next_point));
        vertices.push(next_vertex);
        current_point = next_point;
    }

    // Create edges between vertices
    for i in 0..vertices.len() - 1 {
        let edge = truck_modeling::builder::line(&vertices[i], &vertices[i + 1]);
        edges.push(edge);
    }

    // Create closing edge
    let closing_edge = truck_modeling::builder::line(&vertices.last().unwrap(), &vertices[0]);
    edges.push(closing_edge);

    let wire = truck_modeling::Wire::from_iter(edges.into_iter());

    let face =
        truck_modeling::builder::try_attach_plane(&[wire]).map_err(|e| format!("{:?}", e))?;

    return_model(Arc::new(face), env)
}

/// Create a circle wire in the XY plane
///
/// # Lisp Usage
///
/// ```lisp
/// (circle x y r)
/// ```
///
/// # Examples
///
/// ```lisp
/// (circle 0 0 5)  ;; Create a circle at origin with radius 5
/// (circle 10 20 3)  ;; Create a circle at (10, 20) with radius 3
/// ```
///
/// # Returns
///
/// A model expression representing the created circle wire
#[lisp_fn]
fn circle(args: &[Arc<Expr>], env: Arc<Mutex<Env>>) -> Result<Arc<Expr>, String> {
    assert_arg_count(args, 3)?;

    let x = extract::number(args[0].as_ref())?;
    let y = extract::number(args[1].as_ref())?;
    let radius = extract::number(args[2].as_ref())?;

    let v1 = builder::vertex(Point3::new(x - radius, y, 0.0));
    let v2 = builder::vertex(Point3::new(x, y, 0.0));
    let p3 = Point3::new(x + radius, y, 0.0);

    let edge = truck_modeling::builder::circle_arc(&v1, &v2, p3);

    let wire = truck_modeling::Wire::from(vec![edge]);
    let face =
        truck_modeling::builder::try_attach_plane(&[wire]).map_err(|e| format!("{:?}", e))?;

    return_model(Model::Face(Arc::new(face)), env)
}

#[lisp_fn]
fn linear_extrude(args: &[Arc<Expr>], env: Arc<Mutex<Env>>) -> Result<Arc<Expr>, String> {
    assert_arg_count(args, 2)?;

    let face = extract::face(args[0].as_ref(), &env)?;
    let height = extract::number(args[1].as_ref())?;
    let solid = truck_modeling::builder::tsweep(&*face, truck_modeling::Vector3::unit_z() * height);

    return_model(Arc::new(solid), env)
}

#[lisp_fn]
fn to_mesh(args: &[Arc<Expr>], env: Arc<Mutex<Env>>) -> Result<Arc<Expr>, String> {
    assert_arg_count(args, 1)?;

    let solid = extract::solid(args[0].as_ref(), &env)?;

    let mesh = solid.triangulation(0.01).to_polygon();
    return_model(Arc::new(mesh), env)
}

#[lisp_fn]
fn sandbox(_: &[Arc<Expr>], env: Arc<Mutex<Env>>) -> Result<Arc<Expr>, String> {
    let wire = truck_modeling::Wire::from(vec![]);
    let face =
        truck_modeling::builder::try_attach_plane(&[wire]).map_err(|e| format!("{:?}", e))?;
    let solid = truck_modeling::builder::tsweep(&face, truck_modeling::Vector3::unit_z());
    let mesh = solid.triangulation(0.01).to_polygon();
    return_model(Arc::new(mesh), env)
}

#[allow(dead_code)]
fn debug(wire: &truck_modeling::Wire) {
    let vertex_format = VertexDisplayFormat::Full;
    let _edge_format = EdgeDisplayFormat::Full { vertex_format };
    let wire_format = WireDisplayFormat::VerticesList { vertex_format };
    println!("{:?}", wire.display(wire_format));
}
