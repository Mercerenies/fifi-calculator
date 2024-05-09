// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use fifi::state::WrappedApplicationState;
use fifi::expr::Expr;

#[tauri::command]
fn submit_integer(state: tauri::State<WrappedApplicationState>, value: i64) {
  let mut state = state.lock().expect("poisoned mutex");
  state.main_stack.push(Expr::Atom(value.into()));
  println!("integer: {}", value);
  println!("{:?}", state.main_stack);
}

fn main() {
  tauri::Builder::default()
    .manage(WrappedApplicationState::new())
    .invoke_handler(tauri::generate_handler![submit_integer])
    .run(tauri::generate_context!())
    .expect("error while running tauri application");
}
