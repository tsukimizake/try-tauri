// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]
mod data;
mod lisp;

use data::stl::{FromTauriCmdType, ToTauriCmdType};
use std::io::Read;
use tauri::api::dialog::FileDialogBuilder;

#[tauri::command(rename_all = "snake_case")]
fn from_elm(window: tauri::Window, args: String) -> () {
    println!("to_tauri: {:?}", args);
    match serde_json::from_str(&args).unwrap() {
        ToTauriCmdType::RequestStlFile => {
            read_stl_file(window);
        }
        ToTauriCmdType::RequestCode(path) => {
            println!("path: {:?}", path);
            read_code_file(window, &path);
        }
    }
}

fn read_stl_file(window: tauri::Window) {
    FileDialogBuilder::new()
        .add_filter("STL Files", &["stl"])
        .pick_file(|file_path| {
            // do something with the optional file path here
            // the file path is `None` if the user closed the dialog
            match file_path {
                Some(path) => {
                    let mut input = std::fs::File::open(path).unwrap();

                    let mut buf: Vec<u8> = Vec::new();
                    input.read_to_end(&mut buf).unwrap();
                    to_elm(window, FromTauriCmdType::StlBytes(buf));
                }
                None => {
                    println!("User closed the dialog without selecting a file");
                }
            }
        })
}

fn read_code_file(window: tauri::Window, path: &str) {
    if let Ok(code) = std::fs::read_to_string(path) {
        window
            .emit("tauri_msg", FromTauriCmdType::Code(code))
            .unwrap();
    } else {
        println!("Failed to read code file");
    }
}

// TODO data should be a struct with tag
#[tauri::command]
fn to_elm(window: tauri::Window, cmd: FromTauriCmdType) {
    match window.emit("tauri_msg", cmd) {
        Ok(_) => println!("event sent successfully"),
        Err(e) => println!("failed to send event: {}", e),
    }
}

fn main() {
    let _expr =
        lisp::run_file("(define (sum n) (if (< n 1) n (+ n (sum (- n 1))))) (sum 0)").unwrap();
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
        .invoke_handler(tauri::generate_handler![from_elm, to_elm])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
