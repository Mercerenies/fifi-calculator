
//! Tauri command-like functions.

use super::{ApplicationState, UndoDirection};
use super::validation::{Validator, validate};
use super::events::show_error;
use crate::command::{CommandContext, CommandOutput};
use crate::command::dispatch::CommandDispatchTable;
use crate::command::options::CommandOptions;
use crate::error::Error;
use crate::errorlist::ErrorList;
use crate::expr::Expr;
use crate::expr::var::Var;
use crate::expr::simplifier::default_simplifier;
use crate::expr::number::Number;
use crate::stack::base::StackLike;
use crate::stack::keepable::KeepableStack;

use std::str::FromStr;
use std::fmt::Display;

/// Parses the argument as a real number and pushes it onto the main
/// stack. In case of parse error, the stack is unchanged.
pub fn submit_number(
  state: &mut ApplicationState,
  app_handle: &tauri::AppHandle,
  value: &str,
) -> Result<(), Error> {
  let number = Number::from_str(value)?;
  state.undo_stack_mut().push_cut();
  state.main_stack_mut().push(Expr::from(number));

  state.send_all_updates(app_handle)?;
  Ok(())
}

/// Parses the argument as an arbitrary [`Expr`] and pushes it onto
/// the main stack. In case of parse error, the stack is unchanged.
pub fn submit_expr(
  state: &mut ApplicationState,
  app_handle: &tauri::AppHandle,
  value: &str,
) -> Result<(), Error> {
  let mut errors = ErrorList::new();
  let simplifier = default_simplifier();
  let language_mode = &state.display_settings().language_mode;
  let expr = language_mode.parse(value)?;
  let expr = simplifier.simplify_expr(expr, &mut errors);
  state.undo_stack_mut().push_cut();
  state.main_stack_mut().push(expr);

  state.send_all_updates(app_handle)?;
  Ok(())
}

/// Runs the given (nullary) math command from the command dispatch
/// table.
pub fn run_math_command(
  state: &mut ApplicationState,
  app_handle: &tauri::AppHandle,
  command_table: &CommandDispatchTable,
  command_name: &str,
  opts: CommandOptions,
) -> Result<(), Error> {
  let command = command_table.get(command_name)?;
  let context = CommandContext {
    opts,
    simplifier: default_simplifier(),
  };
  let output = command.run_command(state, vec![], &context)?; // TODO: Args
  handle_command_output(app_handle, &output)?;

  state.send_all_updates(app_handle)?;
  Ok(())
}

/// Runs the given undo action.
pub fn perform_undo_action(
  state: &mut ApplicationState,
  app_handle: &tauri::AppHandle,
  direction: UndoDirection,
) -> Result<(), Error> {
  // We disable the undo/redo on-screen buttons if there's no action
  // to perform. But the user can still use keyboard shortcuts to
  // trigger them anyway, so these actions can fail. If they do, they
  // perform no operations and harmlessly fail, so we can ignore Err
  // here.
  let _ = match direction {
    UndoDirection::Undo => state.undo(),
    UndoDirection::Redo => state.redo(),
  };

  state.send_all_updates(app_handle)?;
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

/// Performs a variable substitution on the top stack element.
pub fn substitute_variable(
  state: &mut ApplicationState,
  app_handle: &tauri::AppHandle,
  variable_name: String,
  new_value: &str,
  opts: CommandOptions,
) -> Result<(), Error> {
  let mut errors = ErrorList::new();
  state.undo_stack_mut().push_cut();

  let language_mode = &state.display_settings().language_mode;
  let variable_name = Var::try_from(variable_name).map_err(Error::custom_error)?;
  let new_value = language_mode.parse(new_value)?;

  let simplifier = default_simplifier();
  let mut stack = KeepableStack::new(state.main_stack_mut(), opts.keep_modifier);
  let expr = stack.pop()?;
  let expr = expr.substitute_var(variable_name, new_value);
  let expr = simplifier.simplify_expr(expr, &mut errors);
  stack.push(expr);

  handle_error_list(app_handle, errors)?;
  state.send_all_updates(app_handle)?;
  Ok(())
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
  if !command_output.errors.is_empty() {
    show_error(app_handle, format!("Error: {}", command_output.errors[0]))
  } else {
    Ok(())
  }
}

/// Handles any errors *except* [`tauri::Error`] by displaying them to
/// the user in a notification box. `tauri::Error` values are passed
/// through and not handled.
pub fn handle_non_tauri_errors(app_handle: &tauri::AppHandle, value: Result<(), Error>) -> Result<(), tauri::Error> {
  match value {
    Ok(()) => Ok(()),
    Err(Error::TauriError(e)) => Err(e),
    Err(other) => {
      show_error(app_handle, format!("Error: {}", other))?;
      Ok(())
    }
  }
}
