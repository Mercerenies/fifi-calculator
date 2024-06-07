
//! [`UndoableChange`] implementations for the application's
//! [`UndoableState`].

use crate::undo::UndoableChange;
use crate::expr::Expr;
use crate::expr::var::Var;
use crate::stack::base::RandomAccessStackLike;
use super::UndoableState;

/// `UndoableChange` that pushes a single value onto the stack at the
/// given position.
#[derive(Clone, Debug)]
pub struct PushExprChange {
  index: usize,
  expr: Expr,
}

/// `UndoableChange` that pops a single value off the stack, not
/// necessarily the top one.
#[derive(Clone, Debug)]
pub struct PopExprChange {
  index: usize,
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

/// `UndoableChange` that replaces a variable binding's presence in
/// the state's variable table. This change can be used to add,
/// remove, or update bindings.
#[derive(Clone, Debug)]
pub struct UpdateVarChange {
  var: Var,
  old_value: Option<Expr>,
  new_value: Option<Expr>,
}

impl PushExprChange {
  pub fn new(index: usize, expr: Expr) -> Self {
    Self { index, expr }
  }
}

impl PopExprChange {
  pub fn new(index: usize, expr: Expr) -> Self {
    Self { index, expr }
  }
}

impl ReplaceExprChange {
  pub fn new(index: i64, old_expr: Expr, new_expr: Expr) -> Self {
    Self { index, old_expr, new_expr }
  }
}

impl UpdateVarChange {
  pub fn new(var: Var, old_value: Option<Expr>, new_value: Option<Expr>) -> Self {
    Self { var, old_value, new_value }
  }

  pub fn create_var(var: Var, new_value: Expr) -> Self {
    Self { var, old_value: None, new_value: Some(new_value) }
  }

  pub fn destroy_var(var: Var, old_value: Expr) -> Self {
    Self { var, old_value: Some(old_value), new_value: None }
  }

  pub fn update_var(var: Var, old_value: Expr, new_value: Expr) -> Self {
    Self { var, old_value: Some(old_value), new_value: Some(new_value) }
  }
}

impl UndoableChange<UndoableState> for PushExprChange {
  fn play_forward(&self, state: &mut UndoableState) {
    let _ = state.main_stack_mut().insert(self.index, self.expr.clone());
  }

  fn play_backward(&self, state: &mut UndoableState) {
    let _ = state.main_stack_mut().pop_nth(self.index);
  }

  fn undo_summary(&self) -> String {
    format!("{:?}", self)
  }
}

impl UndoableChange<UndoableState> for PopExprChange {
  fn play_forward(&self, state: &mut UndoableState) {
    let _ = state.main_stack_mut().pop_nth(self.index);
  }

  fn play_backward(&self, state: &mut UndoableState) {
    let _ = state.main_stack_mut().insert(self.index, self.expr.clone());
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

impl UndoableChange<UndoableState> for UpdateVarChange {
  fn play_forward(&self, state: &mut UndoableState) {
    let table = state.variable_table_mut();
    match self.new_value.clone() {
      Some(new_value) => table.insert(self.var.clone(), new_value),
      None => table.remove(&self.var),
    };
  }

  fn play_backward(&self, state: &mut UndoableState) {
    let table = state.variable_table_mut();
    match self.old_value.clone() {
      Some(old_value) => table.insert(self.var.clone(), old_value),
      None => table.remove(&self.var),
    };
  }

  fn undo_summary(&self) -> String {
    format!("{:?}", self)
  }
}
