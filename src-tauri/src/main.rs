// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use fifi::error::Error;
use fifi::state::{TauriApplicationState, ApplicationState};
use fifi::command::dispatch::CommandDispatchTable;
use fifi::expr::Expr;

#[tauri::command]
fn submit_integer(
  app_state: tauri::State<TauriApplicationState>,
  app_handle: tauri::AppHandle,
  value: i64,
) -> Result<(), tauri::Error> {
  let mut state = app_state.state.lock().expect("poisoned mutex");
  state.main_stack.push(Expr::Atom(value.into()));
  state.send_refresh_stack_event(&app_handle)?;
  Ok(())
}

#[tauri::command]
fn math_command(
  app_state: tauri::State<TauriApplicationState>,
  app_handle: tauri::AppHandle,
  command_name: &str,
) -> Result<(), tauri::Error> {
  let mut state = app_state.state.lock().expect("poisoned mutex");
  handle_non_tauri_errors(
    run_math_command(&mut state, &app_state.command_table, command_name),
  )?;
  state.send_refresh_stack_event(&app_handle)?;
  Ok(())
}

fn run_math_command(
  state: &mut ApplicationState,
  table: &CommandDispatchTable,
  command_name: &str,
) -> Result<(), Error> {
  let command = table.get(command_name)?;
  command.run_command(state)
}

fn handle_non_tauri_errors(err: Result<(), Error>) -> Result<(), tauri::Error> {
  // This is a temporary solution that simply prints out any non-Tauri
  // error to stderr. In the future, we'll show those errors in the
  // UI. Note that Tauri errors should always be reported back to the
  // Tauri runtime, so we very specifically don't handle those here.
  match err {
    Ok(()) => Ok(()),
    Err(Error::TauriError(e)) => Err(e),
    Err(other) => {
      // TODO Show in UI instead.
      eprintln!("Error: {}", other);
      Ok(())
    }
  }
}

fn main() {
  tauri::Builder::default()
    .manage(TauriApplicationState::new())
    .invoke_handler(tauri::generate_handler![submit_integer, math_command])
    .run(tauri::generate_context!())
    .expect("error while running tauri application");
}
