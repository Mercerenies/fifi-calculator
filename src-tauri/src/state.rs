
//! Backend application state manager.

use crate::stack::Stack;
use crate::expr::Expr;
use crate::events::RefreshStackPayload;
use crate::command::default_dispatch_table;
use crate::command::dispatch::CommandDispatchTable;
use crate::display::DisplaySettings;

use tauri::Manager;

use std::sync::Mutex;

pub struct TauriApplicationState {
  pub state: Mutex<ApplicationState>,
  pub command_table: CommandDispatchTable,
}

#[derive(Default)]
pub struct ApplicationState {
  pub main_stack: Stack<Expr>,
  pub display_settings: DisplaySettings,
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
    let displayed_stack: Vec<String> =
      self.main_stack.iter().map(|expr| self.display_settings.to_html(expr)).collect();
    let payload = RefreshStackPayload { stack: displayed_stack };
    app_handle.emit_all(RefreshStackPayload::EVENT_NAME, payload)
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
