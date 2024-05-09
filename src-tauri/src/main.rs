// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use fifi::state::WrappedApplicationState;

#[tauri::command]
fn submit_integer(_state: tauri::State<WrappedApplicationState>, value: i64) {
  println!("integer: {}", value);
}

fn main() {
  tauri::Builder::default()
    .manage(WrappedApplicationState::new())
    .invoke_handler(tauri::generate_handler![submit_integer])
    .run(tauri::generate_context!())
    .expect("error while running tauri application");
}
