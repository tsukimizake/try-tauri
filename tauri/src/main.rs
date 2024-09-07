// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]
mod elm;
mod lisp;

use elm::{FromTauriCmdType, ToTauriCmdType};
use std::io::Read;
use std::sync::{Arc, Mutex};
use stl_io::IndexedMesh;

struct SharedState {
    pub stl: Mutex<Option<IndexedMesh>>,
    pub code: Mutex<String>,
    pub lisp_env: Arc<Mutex<lisp::env::Env>>,
}

impl SharedState {
    fn default() -> Self {
        Self {
            stl: Mutex::default(),
            code: Mutex::default(),
            lisp_env: lisp::eval::initial_env(),
        }
    }
}

#[tauri::command(rename_all = "snake_case")]
fn from_elm(
    window: tauri::Window,
    state: tauri::State<SharedState>,
    args: String,
) -> Result<(), String> {
    println!("to_tauri: {:?}", args);
    match serde_json::from_str(&args).unwrap() {
        ToTauriCmdType::RequestStlFile(path) => {
            if let Some(buf) = read_stl_file(&path) {
                if let Ok(mesh) = stl_io::read_stl(&mut std::io::Cursor::new(&buf)) {
                    state.stl.lock().unwrap().replace(mesh);
                    to_elm(window, FromTauriCmdType::StlBytes(buf));
                }
            }
            Ok(())
        }
        ToTauriCmdType::RequestCode(path) => {
            read_code_file(window, state, &path);
            Ok(())
        }
        ToTauriCmdType::RequestEval => {
            let code = state.code.lock().unwrap().clone();
            let result = match lisp::run_file(&code, state.lisp_env.clone()) {
                Ok(expr) => FromTauriCmdType::EvalOk(expr.format()),
                Err(err) => FromTauriCmdType::EvalError(err),
            };
            to_elm(window, result);
            Ok(())
        }
    }
}

fn read_stl_file(path: &str) -> Option<Vec<u8>> {
    let mut input = std::fs::File::open(path).unwrap();
    let mut buf: Vec<u8> = Vec::new();
    input.read_to_end(&mut buf).unwrap();
    Some(buf)
}

fn read_code_file(window: tauri::Window, state: tauri::State<SharedState>, path: &str) {
    if let Ok(code) = std::fs::read_to_string(path) {
        let mut r = state.code.lock().unwrap();
        r.clear();
        r.push_str(&code);
        to_elm(window, FromTauriCmdType::Code(code));
    } else {
        println!("Failed to read code file");
    }
}

#[tauri::command]
fn to_elm(window: tauri::Window, cmd: FromTauriCmdType) {
    match window.emit("tauri_msg", cmd) {
        Ok(_) => println!("event sent successfully"),
        Err(e) => println!("failed to send event: {}", e),
    }
}

fn main() {
    // the target would typically be a file
    let mut target = vec![];
    // elm_rs provides a macro for conveniently creating an Elm module with everything needed
    elm_rs::export!("Bindings", &mut target, {
        encoders: [ToTauriCmdType, FromTauriCmdType],
        decoders: [ToTauriCmdType, FromTauriCmdType],
    })
    .unwrap();
    let output = String::from_utf8(target).unwrap();

    std::fs::write("../src/elm/Bindings.elm", output).unwrap();

    tauri::Builder::default()
        .manage(SharedState::default())
        .invoke_handler(tauri::generate_handler![from_elm, to_elm])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
