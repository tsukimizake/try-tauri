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
use truck_modeling::builder;

fn add_stl_to_env(mesh: PolygonMesh, env: &Arc<Mutex<Env>>) -> Arc<Expr> {
    let stl_obj = Arc::new(mesh);
    let stl_id = env.lock().unwrap().insert_model(stl_obj);
    Arc::new(Expr::model(stl_id))
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
    if let Err(e) = assert_arg_count(args, 1) {
        return Err(e);
    }
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

/// Create a vertex at the specified coordinates
///
/// # Lisp Usage
///
/// This function is available as both `vertex` and `p` in Lisp:
///
/// ```lisp
/// (vertex x y z)  ;; Using the full name with 3 coordinates
/// (vertex x y)    ;; Using the full name with 2 coordinates (z=0)
/// (p x y z)       ;; Using the shorthand alias with 3 coordinates
/// (p x y)         ;; Using the shorthand alias with 2 coordinates (z=0)
/// ```
///
/// # Examples
///
/// ```lisp
/// (vertex 1 2 3)  ;; Create a vertex at (1, 2, 3)
/// (p 0 0 0)       ;; Create a vertex at origin using the shorthand
/// (p 10 5)        ;; Create a vertex at (10, 5, 0) - z defaults to 0
///
/// ;; Create multiple vertices
/// (define v1 (p 0 0 0))
/// (define v2 (p 1 0))    ;; z defaults to 0
/// (define v3 (p 0 1 0))
/// ```
///
/// # Returns
///
/// A model expression representing the created vertex
#[lisp_fn("p")]
fn vertex(args: &[Arc<Expr>], env: Arc<Mutex<Env>>) -> Result<Arc<Expr>, String> {
    // Accept either 2 or 3 arguments
    if args.len() != 2 && args.len() != 3 {
        return Err(format!(
            "vertex: expected 2 or 3 arguments, got {}",
            args.len()
        ));
    }

    // Extract x, y coordinates from arguments using the utility function
    let mut coords = args
        .iter()
        .map(|expr| extract::number(expr.as_ref()))
        .collect::<Result<Vec<_>, String>>()?;

    // If only 2 arguments were provided, add a default z-coordinate of 0.0
    if coords.len() == 2 {
        coords.push(0.0);
    }

    // Create a new vertex with the given coordinates
    let point = truck_modeling::Point3::new(coords[0], coords[1], coords[2]);
    let vertex = truck_modeling::Vertex::new(point);

    // Wrap in our type-safe model
    let vertex_model = Model::Vertex(Arc::new(vertex));

    // Add the vertex to the environment and return it as an expression
    let vertex_id = env.lock().unwrap().insert_model(vertex_model);
    Ok(Arc::new(Expr::model(vertex_id)))
}

/// Create a line between two vertices
///
/// # Lisp Usage
///
/// ```lisp
/// (line vertex1 vertex2)
/// ```
///
/// # Examples
///
/// ```lisp
/// (define v1 (p 0 0 0))
/// (define v2 (p 1 1 1))
/// (line v1 v2)  ;; Create a line from origin to (1,1,1)
///
/// ;; Create a square using lines
/// (define v1 (p 0 0 0))
/// (define v2 (p 1 0 0))
/// (define v3 (p 1 1 0))
/// (define v4 (p 0 1 0))
/// (define l1 (line v1 v2))
/// (define l2 (line v2 v3))
/// (define l3 (line v3 v4))
/// (define l4 (line v4 v1))
/// ```
///
/// # Returns
///
/// A model expression representing the created line (edge)
#[lisp_fn]
fn line(args: &[Arc<Expr>], env: Arc<Mutex<Env>>) -> Result<Arc<Expr>, String> {
    if let Err(e) = assert_arg_count(args, 2) {
        return Err(e);
    }

    // Extract vertices from arguments using the utility function
    let vertices = args
        .iter()
        .map(|expr| extract::vertex(expr.as_ref(), &env))
        .collect::<Result<Vec<_>, String>>()?;

    // Create a new edge with the given vertices
    let edge = truck_modeling::builder::line(&vertices[0], &vertices[1]);

    // Wrap in our type-safe model
    let edge_model = Model::Edge(Arc::new(edge));

    // Add the edge to the environment and return it as an expression
    let edge_id = env.lock().unwrap().insert_model(edge_model);
    Ok(Arc::new(Expr::model(edge_id)))
}

/// turtle_sketch to create a face from a sequence of vertices.
///
/// # Lisp Usage
///
/// ```lisp
/// (turtle vertex1 vertex2 vertex3 ...)
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
    if args.is_empty() {
        return Err("turtle: expected at least one vertex".to_string());
    }

    // Extract vertices from arguments using the utility function
    let vertices = args
        .iter()
        .map(|expr| extract::vertex(expr.as_ref(), &env))
        .collect::<Result<Vec<_>, String>>()?;

    // Need at least three vertices to make a face
    if vertices.len() < 3 {
        return Err("turtle: expected at least three vertices to create a face".to_string());
    }

    // Create edges using turtle graphics approach
    let mut edges = Vec::new();

    // Start at the first vertex position
    let start_point = vertices[0].point();
    let mut current_point = start_point;
    let mut current_vertex = Arc::new(truck_modeling::Vertex::new(current_point));

    // Create a list of points to track our path
    let mut path_points = vec![current_point];

    // For each vertex after the first, treat it as a movement vector
    for i in 1..vertices.len() {
        // Get the movement vector from the next vertex
        let movement = vertices[i].point();

        // Add the movement to the current position
        let next_point = truck_modeling::Point3::new(
            current_point.x + movement.x,
            current_point.y + movement.y,
            current_point.z + movement.z,
        );

        // Create a new vertex at the new position
        let next_vertex = Arc::new(truck_modeling::Vertex::new(next_point));

        // Create an edge from current to next
        let edge = truck_modeling::builder::line(&current_vertex, &next_vertex);
        edges.push(edge);

        // Update current position and vertex
        current_point = next_point;
        current_vertex = next_vertex;
        path_points.push(current_point);
    }

    // Close the loop by connecting back to the start
    if path_points.len() >= 3 {
        let last_vertex = Arc::new(truck_modeling::Vertex::new(
            path_points.last().unwrap().clone(),
        ));
        let first_vertex = Arc::new(truck_modeling::Vertex::new(
            path_points.first().unwrap().clone(),
        ));
        let closing_edge = truck_modeling::builder::line(&last_vertex, &first_vertex);
        edges.push(closing_edge);
    }

    // Create a wire from the edges
    let wire = truck_modeling::Wire::from_iter(edges.into_iter());

    // Create a face from the wire
    let face = truck_modeling::builder::try_attach_plane(&[wire]).unwrap();

    let face_model = Model::Face(Arc::new(face));

    // Add the face to the environment and return it as an expression
    let face_id = env.lock().unwrap().insert_model(face_model);
    Ok(Arc::new(Expr::model(face_id)))
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
    if let Err(e) = assert_arg_count(args, 3) {
        return Err(e);
    }

    // Extract x, y, radius from arguments
    let x = extract::number(args[0].as_ref())?;
    let y = extract::number(args[1].as_ref())?;
    let radius = extract::number(args[2].as_ref())?;

    let edge = truck_modeling::builder::circle_arc(
        &builder::vertex(Point3::new(x - radius, y, 0.0)),
        &builder::vertex(Point3::new(x + radius, y, 0.0)),
        Point3::new(x, y + radius, 0.0),
    );

    // Then create a wire from the curve
    let wire = truck_modeling::Wire::from(vec![edge]);

    // Wrap in our type-safe model
    let wire_model = Model::Wire(Arc::new(wire));

    // Add the wire to the environment and return it as an expression
    let wire_id = env.lock().unwrap().insert_model(wire_model);
    Ok(Arc::new(Expr::model(wire_id)))
}

#[lisp_fn]
fn linear_extrude(args: &[Arc<Expr>], env: Arc<Mutex<Env>>) -> Result<Arc<Expr>, String> {
    if let Err(e) = assert_arg_count(args, 2) {
        return Err(e);
    }

    // Extract the face and height from arguments using the utility function
    let face = extract::face(args[0].as_ref(), &env)?;
    let height = extract::number(args[1].as_ref())?;

    // Extrude the face to create a solid
    // Need to dereference the Arc to get the Face directly
    // Also need to call unit_z() as it's a function
    let solid = truck_modeling::builder::tsweep(&*face, truck_modeling::Vector3::unit_z() * height);

    // Wrap in our type-safe model
    let solid_model = Model::Solid(Arc::new(solid));

    // Add the solid to the environment and return it as an expression
    let solid_id = env.lock().unwrap().insert_model(solid_model);
    Ok(Arc::new(Expr::model(solid_id)))
}

#[lisp_fn]
fn to_mesh(args: &[Arc<Expr>], env: Arc<Mutex<Env>>) -> Result<Arc<Expr>, String> {
    if let Err(e) = assert_arg_count(args, 1) {
        return Err(e);
    }

    // Extract the solid from arguments using the utility function
    let solid = extract::solid(args[0].as_ref(), &env)?;

    // Convert the solid to a polygon mesh
    let mesh = solid.triangulation(0.01).to_polygon();

    // Wrap in our type-safe model
    let mesh_model = Model::Mesh(Arc::new(mesh));

    // Add the mesh to the environment and return it as an expression
    let mesh_id = env.lock().unwrap().insert_model(mesh_model);
    Ok(Arc::new(Expr::model(mesh_id)))
}
