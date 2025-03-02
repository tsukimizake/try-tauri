use crate::lisp::env::Model;
use std::fs::File;
use std::path::Path;
use truck_polymesh::stl::StlType;

pub fn save_stl_file(model: &Model, filepath: &str) -> Result<(), String> {
    match model {
        Model::Mesh(mesh) => {
            let path = Path::new(filepath);
            let mut file =
                File::create(path).map_err(|e| format!("Failed to create file: {}", e))?;
            truck_polymesh::stl::write(&**mesh, &mut file, StlType::Binary)
                .map_err(|e| format!("Failed to write STL: {}", e))
        }
        _ => Err(format!("Model is not a mesh type")),
    }
}
