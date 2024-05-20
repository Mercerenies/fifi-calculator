
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

/// `UndoableChange` that replaces a single value on the stack with
/// another value.
#[derive(Clone, Debug)]
pub struct ReplaceExprChange {
  index: i64,
  old_expr: Expr,
  new_expr: Expr,
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

impl ReplaceExprChange {
  pub fn new(index: i64, old_expr: Expr, new_expr: Expr) -> Self {
    Self { index, old_expr, new_expr }
  }
}

impl UndoableChange<UndoableState> for PushExprChange {
  fn play_forward(&self, state: &mut UndoableState) {
    state.main_stack_mut().push(self.expr.clone());
  }

  fn play_backward(&self, state: &mut UndoableState) {
    state.main_stack_mut().pop_and_discard();
  }

  fn undo_summary(&self) -> String {
    format!("{:?}", self)
  }
}

impl UndoableChange<UndoableState> for PopExprChange {
  fn play_forward(&self, state: &mut UndoableState) {
    state.main_stack_mut().pop_and_discard();
  }

  fn play_backward(&self, state: &mut UndoableState) {
    state.main_stack_mut().push(self.expr.clone());
  }

  fn undo_summary(&self) -> String {
    format!("{:?}", self)
  }
}

impl UndoableChange<UndoableState> for ReplaceExprChange {
  fn play_forward(&self, state: &mut UndoableState) {
    // There should be no errors if we're undoing the right state, but
    // ignore any that occur, per UndoableChange's contract.
    let _ = state.main_stack_mut().mutate(self.index, |e| {
      *e = self.new_expr.clone();
    });
  }

  fn play_backward(&self, state: &mut UndoableState) {
    // There should be no errors if we're undoing the right state, but
    // ignore any that occur, per UndoableChange's contract.
    let _ = state.main_stack_mut().mutate(self.index, |e| {
      *e = self.old_expr.clone()
    });
  }

  fn undo_summary(&self) -> String {
    format!("{:?}", self)
  }
}
