
//! [`UndoableChange`] implementations for the application's
//! [`UndoableState`].

use crate::undo::UndoableChange;
use crate::expr::Expr;
use super::UndoableState;

/// `UndoableChange` that pushes a single value onto the stack.
#[derive(Clone, Debug)]
pub struct PushExprChange {
  expr: Expr,
}

/// `UndoableChange` that pops a single value off the stack.
#[derive(Clone, Debug)]
pub struct PopExprChange {
  expr: Expr,
}

impl PushExprChange {
  pub fn new(expr: Expr) -> Self {
    Self { expr }
  }
}

impl PopExprChange {
  pub fn new(expr: Expr) -> Self {
    Self { expr }
  }
}

impl UndoableChange<UndoableState> for PushExprChange {
  fn play_forward(&self, state: &mut UndoableState) {
    state.main_stack_mut().push(self.expr.clone());
  }

  fn play_backward(&self, state: &mut UndoableState) {
    state.main_stack_mut().pop();
  }

  fn undo_summary(&self) -> String {
    format!("{:?}", self)
  }
}

impl UndoableChange<UndoableState> for PopExprChange {
  fn play_forward(&self, state: &mut UndoableState) {
    state.main_stack_mut().pop();
  }

  fn play_backward(&self, state: &mut UndoableState) {
    state.main_stack_mut().push(self.expr.clone());
  }
}
