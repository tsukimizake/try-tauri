// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]
mod data;
mod lisp;

use data::stl::StlBytes;
use std::io::Read;
use tauri::api::dialog::FileDialogBuilder;

#[tauri::command]
fn read_stl_file(window: tauri::Window) -> () {
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
                    test_app_handle(window, buf);
                }
                None => {
                    println!("User closed the dialog without selecting a file");
                }
            }
        })
}

// TODO data should be a struct with tag
#[tauri::command]
fn test_app_handle(window: tauri::Window, data: Vec<u8>) {
    match window.emit("tauri_msg", StlBytes { bytes: data }) {
        Ok(_) => println!("event sent successfully"),
        Err(e) => println!("failed to send event: {}", e),
    }
}

fn main() {
    // the target would typically be a file
    let mut target = vec![];
    // elm_rs provides a macro for conveniently creating an Elm module with everything needed
    elm_rs::export!("Bindings", &mut target, {
        encoders: [StlBytes],
        decoders: [StlBytes],
    })
    .unwrap();
    let output = String::from_utf8(target).unwrap();

    std::fs::write("../src/elm/Bindings.elm", output).unwrap();

    tauri::Builder::default()
        .invoke_handler(tauri::generate_handler![read_stl_file, test_app_handle])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
