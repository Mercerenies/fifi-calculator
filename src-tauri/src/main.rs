// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use fifi::command::options::CommandOptions;
use fifi::state::tauri_command::{self, handle_non_tauri_errors};
use fifi::state::validation::Validator;
use fifi::state::{TauriApplicationState, UndoDirection};

#[tauri::command]
fn run_math_command(
    app_state: tauri::State<TauriApplicationState>,
    app_handle: tauri::AppHandle,
    command_name: &str,
    args: Vec<String>,
    opts: CommandOptions,
) -> Result<(), tauri::Error> {
    let mut state = app_state.state.lock().expect("poisoned mutex");
    let function_table = &app_state.function_table;
    let command_table = &app_state.command_table;
    handle_non_tauri_errors(
        &app_handle,
        tauri_command::run_math_command(
            &mut state,
            function_table,
            &app_handle,
            command_table,
            command_name,
            args,
            opts,
        ),
    )
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
    app_handle: tauri::AppHandle,
    value: &str,
    validator: Validator,
) -> Result<bool, tauri::Error> {
    tauri_command::validate_value(&app_handle, value.to_owned(), validator)
}

fn main() {
    tauri::Builder::default()
        .plugin(tauri_plugin_os::init())
        .plugin(tauri_plugin_shell::init())
        .manage(TauriApplicationState::with_default_tables())
        .invoke_handler(tauri::generate_handler![
            run_math_command,
            perform_undo_action,
            validate_stack_size,
            validate_value
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
