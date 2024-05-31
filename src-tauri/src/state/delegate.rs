
//! [`StackDelegate`] implementation that pushes stack changes to an
//! `UndoStack<UndoableState>`.

use super::UndoableState;
use super::undo::{PushExprChange, PopExprChange, ReplaceExprChange};
use crate::undo::UndoStack;
use crate::stack::StackDelegate;
use crate::expr::Expr;

#[derive(Debug)]
pub struct UndoingDelegate<'a> {
  undo_stack: &'a mut UndoStack<UndoableState>,
}

impl<'a> UndoingDelegate<'a> {
  pub fn new(undo_stack: &'a mut UndoStack<UndoableState>) -> Self {
    Self { undo_stack }
  }
}

impl<'a> StackDelegate<Expr> for UndoingDelegate<'a> {
  fn on_push(&mut self, index: usize, new_value: &Expr) {
    self.undo_stack.push_change(PushExprChange::new(index, new_value.clone()));
  }

  fn on_pop(&mut self, index: usize, old_value: &Expr) {
    self.undo_stack.push_change(PopExprChange::new(index, old_value.clone()));
  }

  fn on_mutate(&mut self, index: i64, old_value: &Expr, new_value: &Expr) {
    self.undo_stack.push_change(ReplaceExprChange::new(index, old_value.clone(), new_value.clone()));
  }
}
