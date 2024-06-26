
use super::change::UndoableChange;
use super::error::UndoError;

use std::fmt::{self, Formatter, Debug};

/// A stack of undo-able actions, which can be played backward and
/// then subsequently forward via "Undo" and "Redo" actions.
///
/// The type `S` represents the state of the system. Undo and redo
/// actions require a mutable reference to a value of type `S`, but no
/// other restrictions are placed on what this type must be.
pub struct UndoStack<S> {
  past: Vec<UndoStackValue<S>>,
  future: Vec<UndoStackValue<S>>,
}

enum UndoStackValue<S> {
  Cut,
  Change(Box<dyn UndoableChange<S> + Send + Sync>),
}

impl<S> UndoStack<S> {
  /// A new undo stack, with no actions available to undo or redo.
  pub fn new() -> Self {
    UndoStack {
      past: Vec::new(),
      future: Vec::new(),
    }
  }

  /// Removes all actions from the past, so that `self.has_undos()` is
  /// false.
  pub fn clear_past(&mut self) {
    self.past.clear();
  }

  /// Removes all actions from the future, so that `self.has_redos()`
  /// is false.
  pub fn clear_future(&mut self) {
    self.future.clear();
  }

  /// Removes all actions from both the past and the future, leaving
  /// the undo stack in a state as though it was newly-constructed.
  pub fn clear(&mut self) {
    self.clear_past();
    self.clear_future();
  }

  /// Pushes a cut onto the past stack. Cuts indicate where to stop
  /// undoing and redoing when an action is requested.
  ///
  /// This also clears the future stack, since previously-available
  /// redos are no longer relevant.
  pub fn push_cut(&mut self) {
    self.future.clear();
    self.past.push(UndoStackValue::Cut);
  }

  /// Pushes an [`UndoableChange`] onto the past stack.
  ///
  /// This also clears the future stack, since previously-available
  /// redos are no longer relevant.
  pub fn push_change(&mut self, change: impl UndoableChange<S> + Send + Sync + 'static) {
    self.future.clear();
    self.past.push(UndoStackValue::Change(Box::new(change)));
  }

  /// Performs all changes (via [`UndoableChange::play_backward`]) on
  /// the past stack up to the next cut. Any cuts on top of the past
  /// stack (before any changes) are popped.
  ///
  /// All values popped (whether cuts or changes) during execution of
  /// this method are pushed onto the future stack in reverse order,
  /// so that future redos can play the actions forward.
  ///
  /// Returns `Ok` if any actions were undone, or
  /// [`UndoError::NothingToUndo`] if there was nothing to undo. In
  /// the latter case, any cuts lingering on the undo stack have been
  /// moved to the redo stack, but no `UndoableChange` methods were
  /// called.
  pub fn undo(&mut self, state: &mut S) -> Result<(), UndoError> {
    let is_successful = play_actions(&mut self.past, &mut self.future, |action| {
      action.play_backward(state)
    });
    if is_successful {
      Ok(())
    } else {
      Err(UndoError::NothingToUndo)
    }
  }

  /// Performs all changes (via [`UndoableChange::play_forward`]) on
  /// the future stack up to the next cut. Any cuts on top of the
  /// future stack (before any changes) are popped.
  ///
  /// All values popped (whether cuts or changes) during execution of
  /// this method are pushed onto the past stack in reverse order, so
  /// that future undos can play the actions backward.
  ///
  /// Returns `Ok` if any actions were redone, or
  /// [`UndoError::NothingToRedo`] if there was nothing to redo. In
  /// the latter case, any cuts lingering on the redo stack have been
  /// moved to the undo stack, but no `UndoableChange` methods were
  /// called.
  pub fn redo(&mut self, state: &mut S) -> Result<(), UndoError> {
    let is_successful = play_actions(&mut self.future, &mut self.past, |action| {
      action.play_forward(state)
    });
    if is_successful {
      Ok(())
    } else {
      Err(UndoError::NothingToRedo)
    }
  }

  /// Returns true if there are any changes on the past stack to undo.
  pub fn has_undos(&self) -> bool {
    !self.past.iter().all(UndoStackValue::is_cut)
  }

  /// Returns true if there are any changes on the future stack to
  /// redo.
  pub fn has_redos(&self) -> bool {
    !self.future.iter().all(UndoStackValue::is_cut)
  }
}

/// Plays actions from a stack, pushing onto another stack. Returns
/// true if some actions were successfully played, or false if there
/// was nothing to do.
fn play_actions<S, F>(
  source: &mut Vec<UndoStackValue<S>>,
  dest: &mut Vec<UndoStackValue<S>>,
  mut play_function: F,
) -> bool
where F: FnMut(&dyn UndoableChange<S>) {
  // Pop zero or more cuts before any actions.
  while matches!(source.last(), Some(UndoStackValue::Cut)) {
    let top_cut = source.pop().expect("stack should be nonempty");
    dest.push(top_cut);
  }

  if source.is_empty() {
    // Nothing to do.
    return false;
  }

  // Now play any actions we encounter up to the next cut.
  while matches!(source.last(), Some(UndoStackValue::Change(_))) {
    let Some(UndoStackValue::Change(action)) = source.pop() else {
      panic!("top of stack must be an UndoStackValue::Change");
    };
    play_function(action.as_ref());
    dest.push(UndoStackValue::Change(action));
  }
  true
}

impl<S> UndoStackValue<S> {
  pub fn is_cut(&self) -> bool {
    matches!(self, UndoStackValue::Cut)
  }
}

impl<S> Default for UndoStack<S> {
  fn default() -> Self {
    Self::new()
  }
}

impl<S> Debug for UndoStackValue<S> {
  fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
    match self {
      UndoStackValue::Cut => write!(f, "Cut"),
      UndoStackValue::Change(change) => write!(f, "Change({})", change.undo_summary()),
    }
  }
}

impl<S> Debug for UndoStack<S> {
  fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
    f.debug_struct("UndoStack")
      .field("past", &self.past)
      .field("future", &self.future)
      .finish()
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  #[derive(Clone, Copy, Debug, PartialEq, Eq)]
  struct AddOneAction;

  #[derive(Clone, Copy, Debug, PartialEq, Eq)]
  struct MulTwoAction;

  impl UndoableChange<i32> for AddOneAction {
    fn play_forward(&self, state: &mut i32) {
      *state += 1;
    }
    fn play_backward(&self, state: &mut i32) {
      *state -= 1;
    }
  }

  impl UndoableChange<i32> for MulTwoAction {
    fn play_forward(&self, state: &mut i32) {
      *state *= 2;
    }
    fn play_backward(&self, state: &mut i32) {
      *state /= 2;
    }
  }

  #[test]
  fn test_empty_stack() {
    let mut stack = UndoStack::<i32>::new();
    assert!(!stack.has_undos());
    assert!(!stack.has_redos());
    assert_eq!(stack.undo(&mut 0), Err(UndoError::NothingToUndo));
    assert_eq!(stack.redo(&mut 0), Err(UndoError::NothingToRedo));
  }

  #[test]
  fn test_single_undo_and_redo() {
    let mut stack = UndoStack::<i32>::new();
    stack.push_change(MulTwoAction);
    stack.push_change(AddOneAction);
    stack.push_change(AddOneAction);

    let mut state = 4;
    assert!(stack.has_undos());
    assert!(!stack.has_redos());

    let success = stack.undo(&mut state);
    assert!(success.is_ok());
    assert_eq!(state, 1);
    assert!(!stack.has_undos());
    assert!(stack.has_redos());

    let success = stack.redo(&mut state);
    assert!(success.is_ok());
    assert_eq!(state, 4);
    assert!(stack.has_undos());
    assert!(!stack.has_redos());
  }

  #[test]
  fn test_undo_and_redo_with_one_cut() {
    let mut stack = UndoStack::<i32>::new();
    stack.push_change(MulTwoAction);
    stack.push_cut();
    stack.push_change(AddOneAction);
    stack.push_change(AddOneAction);

    let mut state = 20;
    assert!(stack.has_undos());
    assert!(!stack.has_redos());

    let success = stack.undo(&mut state);
    assert!(success.is_ok());
    assert_eq!(state, 18);
    assert!(stack.has_undos());
    assert!(stack.has_redos());

    let success = stack.undo(&mut state);
    assert!(success.is_ok());
    assert_eq!(state, 9);
    assert!(!stack.has_undos());
    assert!(stack.has_redos());

    let success = stack.undo(&mut state);
    assert!(success.is_err());
    assert_eq!(state, 9);

    let success = stack.redo(&mut state);
    assert!(success.is_ok());
    assert_eq!(state, 18);
    assert!(stack.has_undos());
    assert!(stack.has_redos());

    let success = stack.redo(&mut state);
    assert!(success.is_ok());
    assert_eq!(state, 20);
    assert!(stack.has_undos());
    assert!(!stack.has_redos());

    let success = stack.redo(&mut state);
    assert!(success.is_err());
    assert_eq!(state, 20);
  }

  #[test]
  fn test_undo_and_redo_with_consecutive_cuts() {
    let mut stack = UndoStack::<i32>::new();
    stack.push_cut();
    stack.push_cut();
    stack.push_cut();
    stack.push_change(MulTwoAction);
    stack.push_cut();
    stack.push_cut();
    stack.push_change(AddOneAction);
    stack.push_change(AddOneAction);
    stack.push_cut();
    stack.push_cut();

    let mut state = 20;
    assert!(stack.has_undos());
    assert!(!stack.has_redos());

    let success = stack.undo(&mut state);
    assert!(success.is_ok());
    assert_eq!(state, 18);
    assert!(stack.has_undos());
    assert!(stack.has_redos());

    let success = stack.undo(&mut state);
    assert!(success.is_ok());
    assert_eq!(state, 9);
    assert!(!stack.has_undos());
    assert!(stack.has_redos());

    let success = stack.undo(&mut state);
    assert!(success.is_err());
    assert_eq!(state, 9);

    let success = stack.redo(&mut state);
    assert!(success.is_ok());
    assert_eq!(state, 18);
    assert!(stack.has_undos());
    assert!(stack.has_redos());

    let success = stack.redo(&mut state);
    assert!(success.is_ok());
    assert_eq!(state, 20);
    assert!(stack.has_undos());
    assert!(!stack.has_redos());

    let success = stack.redo(&mut state);
    assert!(success.is_err());
    assert_eq!(state, 20);
  }

  #[test]
  fn test_future_is_clear_after_pushed_cut() {
    let mut stack = UndoStack::<i32>::new();
    let mut state = 64;
    stack.push_change(MulTwoAction);
    stack.push_cut();
    stack.push_change(MulTwoAction);
    assert!(stack.has_undos());
    assert!(!stack.has_redos());

    let success = stack.undo(&mut state);
    assert!(success.is_ok());
    assert_eq!(state, 32);
    assert!(stack.has_undos());
    assert!(stack.has_redos());

    state += 1;
    stack.push_cut();
    assert!(stack.has_undos());
    assert!(!stack.has_redos());
    stack.push_change(AddOneAction);
    assert!(stack.has_undos());
    assert!(!stack.has_redos());

    let success = stack.redo(&mut state);
    assert!(success.is_err());
    assert_eq!(state, 33);

    let success = stack.undo(&mut state);
    assert!(success.is_ok());
    assert_eq!(state, 32);
    assert!(stack.has_undos());
    assert!(stack.has_redos());

    let success = stack.undo(&mut state);
    assert!(success.is_ok());
    assert_eq!(state, 16);
    assert!(!stack.has_undos());
    assert!(stack.has_redos());

    let success = stack.redo(&mut state);
    assert!(success.is_ok());
    assert_eq!(state, 32);
    assert!(stack.has_undos());
    assert!(stack.has_redos());

    let success = stack.redo(&mut state);
    assert!(success.is_ok());
    assert_eq!(state, 33);
    assert!(stack.has_undos());
    assert!(!stack.has_redos());
  }

  #[test]
  fn test_future_is_clear_after_pushed_change() {
    let mut stack = UndoStack::<i32>::new();
    let mut state = 64;
    stack.push_change(MulTwoAction);
    stack.push_cut();
    stack.push_change(MulTwoAction);
    assert!(stack.has_undos());
    assert!(!stack.has_redos());

    let success = stack.undo(&mut state);
    assert!(success.is_ok());
    assert_eq!(state, 32);
    assert!(stack.has_undos());
    assert!(stack.has_redos());

    stack.push_change(AddOneAction);
    assert!(stack.has_undos());
    assert!(!stack.has_redos());
  }

}
