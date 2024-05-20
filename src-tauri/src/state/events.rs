
//! Serializable events that the Rust backend can send (via Tauri) to
//! the frontend.

use serde::Serialize;
use tauri::Manager;

/// Instructs the frontend to re-render the stack elements with the
/// given values.
#[derive(Serialize, Clone, PartialEq, Eq)]
pub struct RefreshStackPayload {
  /// The stack elements, starting from the top.
  pub stack: Vec<String>,
}

/// Instructs the frontend to update the states of the "Undo" and
/// "Redo" buttons.
#[derive(Serialize, Clone, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct UndoAvailabilityPayload {
  pub has_undos: bool,
  pub has_redos: bool,
}

/// Instructs the frontend to render an error message to the user.
#[derive(Serialize, Clone, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct ShowErrorPayload {
  /// The error message to display.
  pub error_message: String,
}

impl RefreshStackPayload {
  pub const EVENT_NAME: &'static str = "refresh-stack";
}

impl UndoAvailabilityPayload {
  pub const EVENT_NAME: &'static str = "refresh-undo-availability";
}

impl ShowErrorPayload {
  pub const EVENT_NAME: &'static str = "show-error";
}

pub fn show_error(app_handle: &tauri::AppHandle, error_message: String) -> tauri::Result<()> {
  app_handle.emit_all(ShowErrorPayload::EVENT_NAME, ShowErrorPayload { error_message })
}
