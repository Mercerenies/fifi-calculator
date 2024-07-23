
//! Cross-platform compatible main function definition.

#[cfg(mobile)]
mod mobile;

use crate::command::CommandContext;
use crate::command::options::CommandOptions;
use crate::state::query::{Query, QueryContext};
use crate::state::tauri_command::{self, handle_non_tauri_errors};
use crate::state::validation::{Validator, ValidationContext};
use crate::state::{TauriApplicationState, UndoDirection};
use crate::state::events::show_error;
use crate::graphics::payload::SerializedGraphicsPayload;
use crate::graphics::response::GraphicsResponse;
use crate::expr::simplifier::default_simplifier;

/// Main entry-point, called from the `fifi` binary crate on desktop
/// platforms.
pub fn run_application() {
  tauri::Builder::default()
    .plugin(tauri_plugin_os::init())
    .plugin(tauri_plugin_shell::init())
    .manage(TauriApplicationState::with_default_tables())
    .invoke_handler(tauri::generate_handler![
      run_math_command,
      render_graphics,
      get_editable_stack_elem,
      perform_undo_action,
      validate_stack_size,
      validate_value,
      query_stack,
    ])
    .run(tauri::generate_context!())
    .expect("error while running tauri application");
}

#[tauri::command]
fn run_math_command(
  app_state: tauri::State<TauriApplicationState>,
  app_handle: tauri::AppHandle,
  command_name: &str,
  args: Vec<String>,
  opts: CommandOptions,
) -> Result<(), tauri::Error> {
  let mut state = app_state.state.lock().expect("poisoned mutex");
  let command_context = CommandContext {
    opts,
    simplifier: default_simplifier(&app_state.function_table),
    units_parser: app_state.units_parser.as_ref(),
  };
  let command_table = &app_state.command_table;
  handle_non_tauri_errors(
    &app_handle,
    tauri_command::run_math_command(
      &mut state,
      command_context,
      &app_handle,
      command_table,
      command_name,
      args,
    ),
  )
}

#[tauri::command]
fn render_graphics(
  app_state: tauri::State<TauriApplicationState>,
  app_handle: tauri::AppHandle,
  payload: SerializedGraphicsPayload,
) -> Result<Option<GraphicsResponse>, tauri::Error> {
  let function_table = &app_state.function_table;
  handle_non_tauri_errors(
    &app_handle,
    tauri_command::render_graphics(
      function_table,
      &app_handle,
      payload,
    ),
  )
}

#[tauri::command]
fn get_editable_stack_elem(
  app_state: tauri::State<TauriApplicationState>,
  app_handle: tauri::AppHandle,
  stack_index: usize,
) -> Result<String, tauri::Error> {
  let mut state = app_state.state.lock().expect("poisoned mutex");
  match tauri_command::get_editable_stack_elem(&mut state, stack_index) {
    Ok(s) => Ok(s),
    Err(err) => {
      show_error(&app_handle, format!("Error: {}", err))?;
      Ok(String::from(""))
    }
  }
}

#[tauri::command]
fn perform_undo_action(
  app_state: tauri::State<TauriApplicationState>,
  app_handle: tauri::AppHandle,
  direction: UndoDirection,
) -> Result<(), tauri::Error> {
  let mut state = app_state.state.lock().expect("poisoned mutex");
  handle_non_tauri_errors(
    &app_handle,
    tauri_command::perform_undo_action(&mut state, &app_handle, direction),
  )
}

#[tauri::command]
fn validate_stack_size(
  app_state: tauri::State<TauriApplicationState>,
  app_handle: tauri::AppHandle,
  expected: usize,
) -> Result<bool, tauri::Error> {
  let state = app_state.state.lock().expect("poisoned mutex");
  tauri_command::validate_stack_size(&state, &app_handle, expected)
}

#[tauri::command]
fn validate_value(
  app_state: tauri::State<TauriApplicationState>,
  app_handle: tauri::AppHandle,
  value: &str,
  validator: Validator,
) -> Result<bool, tauri::Error> {
  let state = app_state.state.lock().expect("poisoned mutex");
  let language_mode = state.display_settings().language_mode();
  let term_parser = state.term_parser();
  let validation_context = ValidationContext {
    units_parser: app_state.units_parser.as_ref(),
    language_mode: language_mode.as_ref(),
    term_parser: &term_parser,
  };
  tauri_command::validate_value(&validation_context, &app_handle, value.to_owned(), validator)
}

#[tauri::command]
fn query_stack(
  app_state: tauri::State<TauriApplicationState>,
  app_handle: tauri::AppHandle,
  query: Query,
) -> Result<bool, tauri::Error> {
  let state = app_state.state.lock().expect("poisoned mutex");
  let term_parser = state.term_parser();
  let query_context = QueryContext {
    units_parser: app_state.units_parser.as_ref(),
    term_parser: &term_parser,
  };
  tauri_command::query_stack(&query_context, &app_handle, &state, &query)
}
