
//! [`UndoableChange`] implementations for the application's
//! [`UndoableState`].

use crate::undo::UndoableChange;
use crate::util::Ellipsis;
use crate::expr::Expr;
use crate::expr::var::Var;
use crate::stack::base::RandomAccessStackLike;
use super::UndoableState;

use std::fmt::{self, Debug, Formatter};

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

/// `UndoableChange` that toggles the value of the given Boolean flag
/// on the state object. A `ToggleFlagChange` shall be its own
/// inverse. That is, since such flags are simply toggling a Boolean
/// value, applying this change forward is equivalent to applying it
/// backward.
pub struct ToggleFlagChange {
  flag_name: String,
  toggle_function: Box<dyn Fn(&mut UndoableState) + Send + Sync>,
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

impl ToggleFlagChange {
  pub fn new<F>(flag_name: impl Into<String>, toggle_function: F) -> Self
  where F: Fn(&mut UndoableState) + Send + Sync + 'static {
    Self {
      flag_name: flag_name.into(),
      toggle_function: Box::new(toggle_function),
    }
  }

  pub fn from_accessor<F>(flag_name: impl Into<String>, accessor: F) -> Self
  where F: Fn(&mut UndoableState) -> &mut bool + Send + Sync + 'static {
    Self::new(flag_name, move |state| {
      let value = accessor(state);
      *value = !*value;
    })
  }

  pub fn from_getter_setter<F, G>(
    flag_name: impl Into<String>,
    getter: F,
    setter: G,
  ) -> Self
  where F: Fn(&UndoableState) -> bool + Send + Sync + 'static,
        G: Fn(&mut UndoableState, bool) + Send + Sync + 'static {
    Self::new(flag_name, move |state| {
      let value = getter(state);
      setter(state, !value);
    })
  }

  /// The name of the flag being toggled. This name has no effect on
  /// semantics but can be useful in debug output and error messages.
  pub fn flag_name(&self) -> &str {
    &self.flag_name
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

impl Debug for ToggleFlagChange {
  fn fmt(&self, f: &mut Formatter) -> fmt::Result {
    f.debug_struct("ToggleFlagChange")
      .field("flag_name", &self.flag_name)
      .field("toggle_function", &Ellipsis)
      .finish()
  }
}

impl UndoableChange<UndoableState> for ToggleFlagChange {
  fn play_forward(&self, state: &mut UndoableState) {
    (self.toggle_function)(state)
  }

  fn play_backward(&self, state: &mut UndoableState) {
    (self.toggle_function)(state)
  }

  fn undo_summary(&self) -> String {
    format!("{:?}", self)
  }
}
