
//! Backend application state manager.

use crate::stack::Stack;
use crate::expr::Expr;
use crate::display::DisplaySettings;

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

}
