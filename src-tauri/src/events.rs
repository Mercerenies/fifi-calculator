
//! Serializable events that the Rust backend can send (via Tauri) to
//! the frontend.

use serde::Serialize;

/// Instructs the frontend to re-render the stack elements with the
/// given values.
#[derive(Serialize, Clone, PartialEq, Eq)]
pub struct RefreshStackPayload {
  /// The stack elements, starting from the top.
  pub stack: Vec<String>,
}

impl RefreshStackPayload {
  pub const EVENT_NAME: &'static str = "refresh-stack";
}
