
//! Backend application state manager.

use crate::stack::Stack;
use crate::expr::Expr;
use crate::events::RefreshStackPayload;
use crate::command::default_dispatch_table;
use crate::command::dispatch::CommandDispatchTable;
use crate::display::DisplaySettings;
use crate::undo::{UndoStack, UndoError};

use tauri::Manager;

use std::sync::Mutex;

pub struct TauriApplicationState {
  pub state: Mutex<ApplicationState>,
  pub command_table: CommandDispatchTable,
}

#[derive(Default)]
pub struct ApplicationState {
  undoable_state: UndoableState,
  undo_stack: UndoStack<UndoableState>,
}

#[derive(Default)]
pub struct UndoableState {
  main_stack: Stack<Expr>,
  display_settings: DisplaySettings,
}

impl TauriApplicationState {
  pub fn new() -> Self {
    Self::default()
  }
}

impl ApplicationState {
  pub fn new() -> Self {
    Self::default()
  }

  pub fn send_refresh_stack_event(&self, app_handle: &tauri::AppHandle) -> tauri::Result<()> {
    let state = &self.undoable_state;
    let displayed_stack: Vec<String> =
      state.main_stack.iter().map(|expr| state.display_settings.to_html(expr)).collect();
    let payload = RefreshStackPayload { stack: displayed_stack };
    app_handle.emit_all(RefreshStackPayload::EVENT_NAME, payload)
  }

  pub fn display_settings(&self) -> &DisplaySettings {
    &self.undoable_state.display_settings
  }

  pub fn display_settings_mut(&mut self) -> &mut DisplaySettings {
    &mut self.undoable_state.display_settings
  }

  pub fn main_stack(&self) -> &Stack<Expr> {
    &self.undoable_state.main_stack
  }

  pub fn main_stack_mut(&mut self) -> &mut Stack<Expr> {
    &mut self.undoable_state.main_stack
  }

  pub fn into_main_stack(self) -> Stack<Expr> {
    self.undoable_state.main_stack
  }

  pub fn undo_stack(&self) -> &UndoStack<UndoableState> {
    &self.undo_stack
  }

  pub fn undo_stack_mut(&mut self) -> &mut UndoStack<UndoableState> {
    &mut self.undo_stack
  }

  pub fn undo(&mut self) -> Result<(), UndoError> {
    self.undo_stack.undo(&mut self.undoable_state)
  }

  pub fn redo(&mut self) -> Result<(), UndoError> {
    self.undo_stack.redo(&mut self.undoable_state)
  }
}

impl Default for TauriApplicationState {
  fn default() -> Self {
    Self {
      state: Mutex::default(),
      command_table: default_dispatch_table(),
    }
  }
}

#[cfg(test)]
pub(crate) mod test_utils {
  use super::*;
  use crate::stack::test_utils::stack_of;

  /// Produces a default state, except that the state's `main_stack`
  /// is equal to `stack` (with the top of the stack being the last
  /// element in the vector).
  pub fn state_for_stack(stack: Vec<i64>) -> ApplicationState {
    let mut state = ApplicationState::new();
    *state.main_stack_mut() = stack_of(stack);
    state
  }
}
