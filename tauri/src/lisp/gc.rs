use super::env::{Env, PolyId};
use super::parser::Expr;
use std::collections::HashSet;
use std::sync::Arc;

pub fn collect_garbage(env: &mut Env) {
    let mut reachable = HashSet::new();
    mark_reachable(env, &mut reachable);
    sweep_unreachable(env, &reachable);
}

fn mark_reachable(env: &Env, reachable: &mut HashSet<PolyId>) {
    // Mark all STL IDs reachable from variables
    for expr in env.vars().values() {
        mark_expr(expr, reachable);
    }

    // Mark all STL IDs reachable from preview list
    for &id in &env.preview_list() {
        reachable.insert(id);
    }

    // Recursively mark parent environment
    if let Some(parent) = &env.parent() {
        mark_reachable(&parent.lock().unwrap(), reachable);
    }
}

fn mark_expr(expr: &Arc<Expr>, reachable: &mut HashSet<PolyId>) {
    match expr.as_ref() {
        Expr::Stl { id, .. } => {
            reachable.insert(*id);
        }
        Expr::List { elements, .. } => {
            for element in elements {
                mark_expr(element, reachable);
            }
        }
        Expr::Quote { expr, .. } => {
            mark_expr(&Arc::new(*expr.clone()), reachable);
        }
        Expr::Clausure { body, .. } => {
            mark_expr(body, reachable);
        }
        _ => {}
    }
}

fn sweep_unreachable(env: &mut Env, reachable: &HashSet<PolyId>) {
    env.retain_polys(|id, _| reachable.contains(id));
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Mutex;
    use truck_polymesh::{Faces, PolygonMesh};

    #[test]
    fn test_define_garbage_collection() {
        let mut env = Env::new();

        // Create test meshes
        let mesh1 = Arc::new(PolygonMesh::new(
            truck_polymesh::StandardAttributes::default(),
            Faces::from_tri_and_quad_faces(vec![], vec![]),
        ));
        let mesh2 = Arc::new(PolygonMesh::new(
            truck_polymesh::StandardAttributes::default(),
            Faces::from_tri_and_quad_faces(vec![], vec![]),
        ));

        // Insert meshes into environment
        let id1 = env.insert_stl(mesh1);
        let id2 = env.insert_stl(mesh2);

        // Define a function that uses mesh1
        env.insert(
            "use_mesh".to_string(),
            Arc::new(Expr::Clausure {
                args: vec!["x".to_string()],
                body: Arc::new(Expr::List {
                    elements: vec![
                        Arc::new(Expr::Symbol {
                            name: "list".to_string(),
                            location: None,
                            trailing_newline: false,
                        }),
                        Arc::new(Expr::Stl {
                            id: id1,
                            location: None,
                            trailing_newline: false,
                        }),
                    ],
                    location: None,
                    trailing_newline: false,
                }),
                env: Arc::new(Mutex::new(Env::new())),
            }),
        );

        // Make id2 reachable through preview list
        env.insert_preview_list(id2);

        // Run garbage collection
        collect_garbage(&mut env);

        // Both meshes should be reachable
        assert!(env.get_stl(id1).is_some(), "mesh1 should be reachable through function definition");
        assert!(env.get_stl(id2).is_some(), "mesh2 should be reachable through preview list");

        // Remove the function definition
        env.vars_mut().clear();

        // Run garbage collection again
        collect_garbage(&mut env);

        // mesh1 should now be collected, but mesh2 still reachable through preview
        assert!(env.get_stl(id1).is_none(), "mesh1 should be collected after removing function");
        assert!(env.get_stl(id2).is_some(), "mesh2 should still be reachable through preview list");
    }

    #[test]
    fn test_stl_garbage_collection() {
        let mut env = Env::new();

        // Create some test meshes
        let mesh1 = Arc::new(PolygonMesh::new(
            truck_polymesh::StandardAttributes::default(),
            Faces::from_tri_and_quad_faces(vec![], vec![]),
        ));
        let mesh2 = Arc::new(PolygonMesh::new(
            truck_polymesh::StandardAttributes::default(),
            Faces::from_tri_and_quad_faces(vec![], vec![]),
        ));
        let mesh3 = Arc::new(PolygonMesh::new(
            truck_polymesh::StandardAttributes::default(),
            Faces::from_tri_and_quad_faces(vec![], vec![]),
        ));

        // Insert meshes into environment
        let id1 = env.insert_stl(mesh1);
        let id2 = env.insert_stl(mesh2);
        let id3 = env.insert_stl(mesh3);

        // Make id1 reachable through a variable
        env.insert(
            "mesh1".to_string(),
            Arc::new(Expr::Stl {
                id: id1,
                location: None,
                trailing_newline: false,
            }),
        );

        // Make id2 reachable through preview list
        env.insert_preview_list(id2);

        // id3 is unreachable

        // Run garbage collection
        collect_garbage(&mut env);

        // Check that reachable meshes are kept
        assert!(env.get_stl(id1).is_some());
        assert!(env.get_stl(id2).is_some());

        // Check that unreachable mesh is collected
        assert!(env.get_stl(id3).is_none());
    }
}
