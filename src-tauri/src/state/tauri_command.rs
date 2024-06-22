
//! Tauri command-like functions.

use super::{ApplicationState, UndoDirection};
use super::validation::{Validator, validate};
use super::events::show_error;
use crate::command::{CommandContext, CommandOutput};
use crate::command::dispatch::CommandDispatchTable;
use crate::command::options::CommandOptions;
use crate::errorlist::ErrorList;
use crate::expr::simplifier::default_simplifier;
use crate::expr::function::table::FunctionTable;
use crate::stack::StackError;
use crate::stack::base::{StackLike, RandomAccessStackLike};
use crate::graphics::payload::SerializedGraphicsPayload;
use crate::graphics::response::GraphicsResponse;

use std::fmt::Display;

/// Runs the given (nullary) math command from the command dispatch
/// table.
pub fn run_math_command(
  state: &mut ApplicationState,
  function_table: &FunctionTable,
  app_handle: &tauri::AppHandle,
  command_table: &CommandDispatchTable,
  command_name: &str,
  args: Vec<String>,
  opts: CommandOptions,
) -> anyhow::Result<()> {
  let command = command_table.get(command_name)?;
  let context = CommandContext {
    opts,
    simplifier: default_simplifier(function_table),
  };
  let output = command.run_command(state, args, &context)?;
  handle_command_output(app_handle, &output)?;

  state.send_all_updates(app_handle, output.force_scroll_down())?;
  Ok(())
}

/// Renders a `graphics` command in the expression language into a set
/// of directives for the frontend to follow.
pub fn render_graphics(
  function_table: &FunctionTable,
  app_handle: &tauri::AppHandle,
  payload: SerializedGraphicsPayload,
) -> anyhow::Result<Option<GraphicsResponse>> {
  let simplifier = default_simplifier(function_table);
  let mut errors = ErrorList::new();

  let payload = payload.try_deserialize()?;
  let response = match payload.compile(&mut errors, simplifier.as_ref(), function_table) {
    Err(err) => {
      err.report_to_user(app_handle)?;
      None
    }
    Ok(response) => {
      Some(response)
    }
  };
  handle_error_list(app_handle, errors)?;
  Ok(response)
}

/// Returns a parser-friendly string representation of the given
/// element on the stack, using the current language mode. Produces a
/// [`StackError`] if the index is out of bounds.
pub fn get_editable_stack_elem(
  state: &mut ApplicationState,
  stack_index: usize,
) -> Result<String, StackError> {
  let elem = state.main_stack().get(stack_index as i64)?;
  Ok(state.display_settings().to_html(elem))
}

/// Runs the given undo action.
pub fn perform_undo_action(
  state: &mut ApplicationState,
  app_handle: &tauri::AppHandle,
  direction: UndoDirection,
) -> anyhow::Result<()> {
  // We disable the undo/redo on-screen buttons if there's no action
  // to perform. But the user can still use keyboard shortcuts to
  // trigger them anyway, so these actions can fail. If they do, they
  // perform no operations and harmlessly fail, so we can ignore Err
  // here.
  let _ = match direction {
    UndoDirection::Undo => state.undo(),
    UndoDirection::Redo => state.redo(),
  };

  state.send_all_updates(app_handle, true)?;
  Ok(())
}

/// Validates the application state's stack size. If the stack is
/// strictly smaller than the desired size, then an error will be
/// issued to the user in the form of a notification. The stack is not
/// modified by this function. Returns true if the stack size is
/// valid.
pub fn validate_stack_size(
  state: &ApplicationState,
  app_handle: &tauri::AppHandle,
  expected_size: usize,
) -> Result<bool, tauri::Error> {
  let validation_passed = match state.main_stack().check_stack_size(expected_size) {
    Ok(()) => true,
    Err(err) => {
      show_error(app_handle, format!("Error: {}", err))?;
      false
    }
  };
  Ok(validation_passed)
}

/// Validates the value against the given validator. Returns true on
/// success. In case of validation failure, this function returns
/// false and reports the error to the user in the form of a
/// notification.
pub fn validate_value(
  app_handle: &tauri::AppHandle,
  value: String,
  validator: Validator,
) -> Result<bool, tauri::Error> {
  let validation_passed = match validate(validator, value) {
    Ok(()) => true,
    Err(err) => {
      show_error(app_handle, format!("Error: {}", err))?;
      false
    }
  };
  Ok(validation_passed)
}

/// Handles errors from the referenced [`ErrorList`] by communicating
/// them to the user.
///
/// Currently, this function only displays the *first* error to the
/// user, for brevity's sake. This behavior may change in the future.
pub fn handle_error_list<E: Display>(app_handle: &tauri::AppHandle, error_list: ErrorList<E>) -> tauri::Result<()> {
  if !error_list.is_empty() {
    show_error(app_handle, format!("Error: {}", error_list.into_vec()[0]))
  } else {
    Ok(())
  }
}

/// Handles errors from the referenced [`CommandOutput`] by
/// communicating them to the user.
///
/// Currently, this function only displays the *first* error to the
/// user, for brevity's sake. This behavior may change in the future.
pub fn handle_command_output(app_handle: &tauri::AppHandle, command_output: &CommandOutput) -> tauri::Result<()> {
  if !command_output.errors().is_empty() {
    show_error(app_handle, format!("Error: {}", command_output.get_error(0)))
  } else {
    Ok(())
  }
}

/// Handles any errors *except* [`tauri::Error`] by displaying them to
/// the user in a notification box. `tauri::Error` values are passed
/// through and not handled.
pub fn handle_non_tauri_errors_or<T>(
  app_handle: &tauri::AppHandle,
  value: anyhow::Result<T>,
  default: T,
) -> Result<T, tauri::Error> {
  match value {
    Ok(value) => Ok(value),
    Err(err) => {
      match err.downcast::<tauri::Error>() {
        Ok(tauri_error) => {
          // Tauri error, so let it propagate.
          Err(tauri_error)
        }
        Err(other_error) => {
          // non-Tauri error, display it and recover.
          show_error(app_handle, format!("Error: {}", other_error))?;
          Ok(default)
        }
      }
    }
  }
}

pub fn handle_non_tauri_errors<T: Default>(
  app_handle: &tauri::AppHandle,
  value: anyhow::Result<T>,
) -> Result<T, tauri::Error> {
  handle_non_tauri_errors_or(app_handle, value, T::default())
}
