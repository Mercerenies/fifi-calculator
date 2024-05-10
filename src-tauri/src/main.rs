// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use fifi::error::Error;
use fifi::state::WrappedApplicationState;
use fifi::expr::Expr;

#[tauri::command]
fn submit_integer(
  state: tauri::State<WrappedApplicationState>,
  app_handle: tauri::AppHandle,
  value: i64,
) -> Result<(), tauri::Error> {
  let mut state = state.lock().expect("poisoned mutex");
  state.main_stack.push(Expr::Atom(value.into()));
  state.send_refresh_stack_event(&app_handle)?;
  Ok(())
}

#[tauri::command]
fn math_command(
  state: tauri::State<WrappedApplicationState>,
  app_handle: tauri::AppHandle,
  command_name: &str,
) -> Result<(), tauri::Error> {
  let mut state = state.lock().expect("poisoned mutex");
  state.send_refresh_stack_event(&app_handle)?;
  Ok(())
}

fn main() {
  tauri::Builder::default()
    .manage(WrappedApplicationState::new())
    .invoke_handler(tauri::generate_handler![submit_integer, math_command])
    .run(tauri::generate_context!())
    .expect("error while running tauri application");
}
