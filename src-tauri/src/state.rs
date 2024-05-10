
//! Backend application state manager.

use crate::stack::Stack;
use crate::error::Error;
use crate::expr::Expr;
use crate::events::RefreshStackPayload;
use crate::command::dispatch::dispatch;
use crate::display::DisplaySettings;

use tauri::Manager;

use std::sync::{Mutex, LockResult, TryLockResult, MutexGuard};

#[derive(Default)]
pub struct WrappedApplicationState {
  state: Mutex<ApplicationState>,
}

#[derive(Default)]
pub struct ApplicationState {
  pub main_stack: Stack<Expr>,
  pub display_settings: DisplaySettings,
}

impl WrappedApplicationState {

  pub fn new() -> Self {
    Self::default()
  }

  pub fn lock(&self) -> LockResult<MutexGuard<'_, ApplicationState>> {
    self.state.lock()
  }

  pub fn try_lock(&self) -> TryLockResult<MutexGuard<'_, ApplicationState>> {
    self.state.try_lock()
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

  pub fn dispatch_and_run_command(&mut self, command_name: &str) -> Result<(), Error> {
    let command = dispatch(command_name)?;
    command.run_command(self)
  }

}
