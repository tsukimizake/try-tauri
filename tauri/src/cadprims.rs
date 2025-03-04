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
use truck_modeling::Solid;
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
/// `(load-stl "path/to/file.stl")`
///
/// # Returns
/// A model expression representing the loaded STL file.
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
/// `(preview model)`
///
/// # Returns
/// The model that was marked for preview
#[lisp_fn]
fn preview(args: &[Arc<Expr>], env: Arc<Mutex<Env>>) -> Result<Arc<Expr>, String> {
    println!("preview: {:?}", args);
    assert_arg_count(args, 1)?;
    match args[0].as_ref() {
        Expr::Model { id, .. } => {
            // Get the model and verify it's a mesh
            let mut env_guard = env.lock().unwrap();
            let model = env_guard
                .get_model(*id)
                .ok_or_else(|| format!("Model with id {} not found", id))?;

            // Check if the model is a mesh
            if let Some(_) = model.as_mesh() {
                // Add to preview list using the same lock
                env_guard.insert_preview_list(*id);

                Ok(args[0].clone())
            } else if let Some(solid) = model.as_solid() {
                let mesh = Arc::new(solid.triangulation(0.01).to_polygon());
                let mesh_clone = mesh.clone();
                let id = env_guard.insert_model(Model::Mesh(mesh));
                env_guard.insert_preview_list(id);

                drop(env_guard);
                return_model(Model::Mesh(mesh_clone), env)
            } else {
                Err("preview: expected solid or mesh model".to_string())
            }
        }

        _ => Err("preview: expected solid or mesh model".to_string()),
    }
}

/// Create a point at the specified coordinates
///
/// # Lisp Usage
/// `(point x y z)` or `(point x y)` or `(p x y z)` or `(p x y)`
/// When z is omitted, it defaults to 0.
///
/// # Examples
/// `(p 1 2 3)` - point at (1,2,3)
/// `(p 10 5)` - point at (10,5,0)
///
/// # Returns
/// A point model
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
/// `(line point1 point2)`
///
/// # Examples
/// `(line (p 0 0) (p 1 1))` - line from origin to (1,1,0)
///
/// # Returns
/// A line (edge) model
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

/// Create a face from a sequence of points using turtle-like movement
///
/// # Lisp Usage
/// `(turtle point1 point2 point3 ...)`
///
/// # Examples
/// `(turtle (p 0 0) (p 1 0) (p 1 1) (p 0 1))` - square face
///
/// # Returns
/// A face model
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

/// Create a circle in the XY plane
///
/// # Lisp Usage
/// `(circle x y radius)`
///
/// # Examples
/// `(circle 0 0 5)` - circle at origin with radius 5
///
/// # Returns
/// A circle face model
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

/// Extrude a face along the Z axis
///
/// # Lisp Usage
/// `(linear-extrude face height)`
///
/// # Examples
/// `(linear-extrude (circle 0 0 5) 10)` - cylinder with radius 5 and height 10
///
/// # Returns
/// A solid model
#[lisp_fn]
fn linear_extrude(args: &[Arc<Expr>], env: Arc<Mutex<Env>>) -> Result<Arc<Expr>, String> {
    assert_arg_count(args, 2)?;

    let face = extract::face(args[0].as_ref(), &env)?;
    let height = extract::number(args[1].as_ref())?;
    let solid = truck_modeling::builder::tsweep(&*face, truck_modeling::Vector3::unit_z() * height);

    return_model(Arc::new(solid), env)
}

#[lisp_fn]
fn sandbox(_: &[Arc<Expr>], env: Arc<Mutex<Env>>) -> Result<Arc<Expr>, String> {
    let wire = truck_modeling::Wire::from(vec![]);
    let face =
        truck_modeling::builder::try_attach_plane(&[wire]).map_err(|e| format!("{:?}", e))?;
    let solid = truck_modeling::builder::tsweep(&face, truck_modeling::Vector3::unit_z());
    return_model(Arc::new(solid), env)
}

#[allow(dead_code)]
fn debug(wire: &truck_modeling::Wire) {
    let vertex_format = VertexDisplayFormat::Full;
    let _edge_format = EdgeDisplayFormat::Full { vertex_format };
    let wire_format = WireDisplayFormat::VerticesList { vertex_format };
    println!("{:?}", wire.display(wire_format));
}

/// Create the intersection of two or more solid models
///
/// # Lisp Usage
/// `(and solid1 solid2 ...)`
///
/// # Examples
/// `(and (linear-extrude (circle 0 0 5) 10) (linear-extrude (circle 5 0 5) 10))` - intersection of two cylinders
///
/// # Returns
/// A solid model representing the intersection
#[lisp_fn]
fn and(args: &[Arc<Expr>], env: Arc<Mutex<Env>>) -> Result<Arc<Expr>, String> {
    assert_arg_count(args, 2..).map_err(|e| format!("and: {}", e))?;

    let solids = args
        .iter()
        .map(|expr| extract::solid(expr.as_ref(), &env))
        .collect::<Result<Vec<_>, String>>()?;

    if solids.len() < 2 {
        return Err("and: expected at least 2 solid models".to_string());
    }

    let mut result: Arc<Solid> = Arc::new((*solids[0]).clone());

    for solid in &solids[1..] {
        result = match truck_shapeops::and(&result, solid, 0.01) {
            Some(solid) => solid.into(),
            None => return Err(format!("Boolean AND operation failed")),
        };
    }
    return_model(result, env)
}

/// Create the union of two or more solid models
///
/// # Lisp Usage
/// `(or solid1 solid2 ...)`
///
/// # Examples
/// `(or (linear-extrude (circle 0 0 5) 10) (linear-extrude (circle 5 0 5) 10))` - union of two cylinders
///
/// # Returns
/// A solid model representing the union
#[lisp_fn]
fn or(args: &[Arc<Expr>], env: Arc<Mutex<Env>>) -> Result<Arc<Expr>, String> {
    assert_arg_count(args, 2..).map_err(|e| format!("or: {}", e))?;

    let solids = args
        .iter()
        .map(|expr| extract::solid(expr.as_ref(), &env))
        .collect::<Result<Vec<_>, String>>()?;

    if solids.len() < 2 {
        return Err("or: expected at least 2 solid models".to_string());
    }

    let mut result: Arc<Solid> = Arc::new((*solids[0]).clone());

    for solid in &solids[1..] {
        result = match truck_shapeops::or(&result, solid, 0.01) {
            Some(solid) => solid.into(),
            None => return Err(format!("Boolean OR operation failed")),
        };
    }
    return_model(result, env)
}

/// Subtract one or more solids from the first solid
///
/// # Lisp Usage
/// `(not base_solid solid_to_subtract1 solid_to_subtract2 ...)`
///
/// # Examples
/// `(not (linear-extrude (circle 0 0 10) 10) (linear-extrude (circle 0 0 5) 20))` - cylinder with a hole
///
/// # Returns
/// A solid model representing the difference
#[lisp_fn]
fn not(args: &[Arc<Expr>], env: Arc<Mutex<Env>>) -> Result<Arc<Expr>, String> {
    assert_arg_count(args, 1)?;

    let solid = extract::solid(args[0].as_ref(), &env)?;

    let mut result: Solid = (*solid).clone();
    result.not();

    return_model(Arc::new(result), env)
}
