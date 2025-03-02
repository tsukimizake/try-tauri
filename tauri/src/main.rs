#![feature(assert_matches)]
// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]
mod elm_interface;
mod lisp;

mod cadprims;
mod stl;
use std::sync::{Arc, Mutex};
use tauri::Emitter; // TODO use emit_to?

use elm_interface::{FromTauriCmdType, ToTauriCmdType};

struct SharedState {
    pub code: Mutex<String>,
    pub lisp_env: Arc<Mutex<lisp::env::Env>>,
}

impl SharedState {
    fn default() -> Self {
        Self {
            code: Mutex::default(),
            lisp_env: Arc::new(Mutex::new(lisp::eval::default_env())),
        }
    }
}

fn init_env(e: Arc<Mutex<lisp::env::Env>>) -> () {
    // TODO 古いstlをgcするために全て捨てているが、evalのキャッシュを持たせる時に捨てすぎでバグるかもしれない
    let mut env = e.lock().unwrap();
    *env = lisp::eval::default_env();
}

#[tauri::command]
fn from_elm(
    window: tauri::Window,
    state: tauri::State<SharedState>,
    args: String,
) -> Result<(), String> {
    println!("to_tauri: {:?}", args);
    match serde_json::from_str(&args).unwrap() {
        ToTauriCmdType::RequestCode(path) => {
            read_code_file(window, state, &path);
            Ok(())
        }
        ToTauriCmdType::RequestEval => {
            let code = state.code.lock().unwrap().clone();
            init_env(state.lisp_env.clone());
            let result = match lisp::run_file(&code, state.lisp_env.clone()) {
                Ok(val) => FromTauriCmdType::EvalOk(val.into()),
                Err(err) => FromTauriCmdType::EvalError(err),
            };
            state.lisp_env.lock().unwrap().collect_garbage();
            to_elm(window, result);
            Ok(())
        }
        ToTauriCmdType::SaveStlFile(stl_id, filepath) => {
            let env_lock = state.lisp_env.lock().unwrap();
            match env_lock.get_model(stl_id) {
                Some(model) => {
                    match stl::save_stl_file(model.as_ref(), &filepath) {
                        Ok(_) => {
                            to_elm(window, FromTauriCmdType::SaveStlFileOk(format!("Successfully saved to {}", filepath)));
                            Ok(())
                        },
                        Err(err) => {
                            to_elm(window, FromTauriCmdType::SaveStlFileError(format!("Error saving file: {}", err)));
                            Ok(())
                        }
                    }
                }
                None => {
                    to_elm(window, FromTauriCmdType::SaveStlFileError(format!("Model ID {} not found", stl_id)));
                    Ok(())
                }
            }
        }
    }
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
        encoders: [
            elm_interface::ToTauriCmdType,
            elm_interface::FromTauriCmdType,
            elm_interface::Evaled,
            elm_interface::Value,
            elm_interface::SerdeStlFaces,
            elm_interface::SerdeStlFace

        ],
        decoders: [
            elm_interface::ToTauriCmdType,
            elm_interface::FromTauriCmdType,
            elm_interface::Evaled,
            elm_interface::Value,
            elm_interface::SerdeStlFaces,
            elm_interface::SerdeStlFace,
        ],
    })
    .unwrap();

    let output = String::from_utf8(target).unwrap();

    std::fs::write("../src/generated/Bindings.elm", output).unwrap();

    tauri::Builder::default()
        .plugin(tauri_plugin_shell::init())
        .plugin(tauri_plugin_dialog::init())
        .manage(SharedState::default())
        .invoke_handler(tauri::generate_handler![from_elm, to_elm])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
