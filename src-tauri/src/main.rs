// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use fifi::error::Error;
use fifi::errorlist::ErrorList;
use fifi::state::{TauriApplicationState, ApplicationState, UndoDirection};
use fifi::state::events::show_error;
use fifi::command::CommandContext;
use fifi::command::options::CommandOptions;
use fifi::command::dispatch::CommandDispatchTable;
use fifi::stack::base::StackLike;
use fifi::expr::simplifier::default_simplifier;
use fifi::expr::Expr;
use fifi::expr::number::Number;

use std::str::FromStr;

#[tauri::command]
fn submit_number(
  app_state: tauri::State<TauriApplicationState>,
  app_handle: tauri::AppHandle,
  value: &str,
) -> Result<(), tauri::Error> {
  let mut state = app_state.state.lock().expect("poisoned mutex");
  handle_non_tauri_errors(&app_handle, parse_and_push_number(&mut state, value))?;
  state.send_refresh_stack_event(&app_handle)?;
  state.send_undo_buttons_event(&app_handle)?;
  Ok(())
}

#[tauri::command]
fn submit_expr(
  app_state: tauri::State<TauriApplicationState>,
  app_handle: tauri::AppHandle,
  value: &str,
) -> Result<(), tauri::Error> {
  let mut state = app_state.state.lock().expect("poisoned mutex");
  handle_non_tauri_errors(&app_handle, parse_and_push_expr(&app_handle, &mut state, value))?;
  state.send_refresh_stack_event(&app_handle)?;
  state.send_undo_buttons_event(&app_handle)?;
  Ok(())
}

#[tauri::command]
fn math_command(
  app_state: tauri::State<TauriApplicationState>,
  app_handle: tauri::AppHandle,
  command_name: &str,
  prefix_argument: Option<i64>,
) -> Result<(), tauri::Error> {
  let mut state = app_state.state.lock().expect("poisoned mutex");
  handle_non_tauri_errors(
    &app_handle,
    run_math_command(&app_handle, &mut state, &app_state.command_table, command_name, prefix_argument),
  )?;
  state.send_refresh_stack_event(&app_handle)?;
  state.send_undo_buttons_event(&app_handle)?;
  Ok(())
}

#[tauri::command]
fn perform_undo_action(
  app_state: tauri::State<TauriApplicationState>,
  app_handle: tauri::AppHandle,
  direction: UndoDirection,
) -> Result<(), tauri::Error> {
  let mut state = app_state.state.lock().expect("poisoned mutex");
  // We disable the undo/redo on-screen buttons if there's no action
  // to perform. But the user can still use keyboard shortcuts to
  // trigger them anyway, so these actions can fail. If they do, they
  // perform no operations and harmlessly fail, so we can ignore Err
  // here.
  let _ = match direction {
    UndoDirection::Undo => state.undo(),
    UndoDirection::Redo => state.redo(),
  };
  state.send_refresh_stack_event(&app_handle)?;
  state.send_undo_buttons_event(&app_handle)?;
  Ok(())
}

fn parse_and_push_number(state: &mut ApplicationState, string: &str) -> Result<(), Error> {
  let number = Number::from_str(string)?;
  let expr = Expr::from(number);
  state.undo_stack_mut().push_cut();
  state.main_stack_mut().push(expr);
  Ok(())
}

fn parse_and_push_expr(
  app_handle: &tauri::AppHandle,
  state: &mut ApplicationState,
  string: &str,
) -> Result<(), Error> {
  let mut errors = ErrorList::new();
  let simplifier = default_simplifier();
  let language_mode = &state.display_settings().language_mode;
  let expr = language_mode.parse(string)?;
  let expr = simplifier.simplify_expr(expr, &mut errors);
  state.undo_stack_mut().push_cut();
  state.main_stack_mut().push(expr);

  if !errors.is_empty() {
    // For now, for brevity, just show the first simplifier error and
    // drop the others. We might revise this later.
    show_error(app_handle, format!("Error: {}", errors.into_vec()[0]))?;
  }

  Ok(())
}

fn run_math_command(
  app_handle: &tauri::AppHandle,
  state: &mut ApplicationState,
  table: &CommandDispatchTable,
  command_name: &str,
  prefix_argument: Option<i64>,
) -> Result<(), Error> {
  let command = table.get(command_name)?;
  let context = CommandContext {
    opts: CommandOptions { argument: prefix_argument, keep_modifier: false }, // TODO: keep_modifier
    simplifier: default_simplifier(),
  };
  let output = command.run_command(state, &context)?;
  if !output.errors.is_empty() {
    // For now, for brevity, just show the first simplifier error and
    // drop the others. We might revise this later.
    show_error(app_handle, format!("Error: {}", output.errors[0]))?;
  }
  Ok(())
}

fn handle_non_tauri_errors(app_handle: &tauri::AppHandle, err: Result<(), Error>) -> Result<(), tauri::Error> {
  match err {
    Ok(()) => Ok(()),
    Err(Error::TauriError(e)) => Err(e),
    Err(other) => {
      show_error(app_handle, format!("Error: {}", other))?;
      Ok(())
    }
  }
}

fn main() {
  tauri::Builder::default()
    .manage(TauriApplicationState::with_default_command_table())
    .invoke_handler(tauri::generate_handler![submit_number, submit_expr, math_command, perform_undo_action])
    .run(tauri::generate_context!())
    .expect("error while running tauri application");
}
