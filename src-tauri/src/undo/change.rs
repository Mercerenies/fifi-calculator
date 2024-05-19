
/// An `UndoableChange` (over some state type `S`) is an action that
/// can be played forward or backward on the state `S`.
pub trait UndoableChange<S> {

  /// Plays the action in the forward direction.
  ///
  /// Note carefully that this action is only played when the "Redo"
  /// action is explicitly requested. Notably, `play_forward` is NOT
  /// called when the action is initially performed as part of the
  /// normal (non-undo-stack) flow of the program.
  fn play_forward(&self, state: &mut S);

  /// Plays the action in the backward direction.
  fn play_backward(&self, state: &mut S);

  /// Debug-friendly summary of the undo action. This method is
  /// optional and is only used in `Debug` impls. The output of this
  /// method is NOT required (or guaranteed) to be stable across
  /// versions and should ONLY be used for debugging purposes.
  fn undo_summary(&self) -> String {
    "UndoableChange".to_string()
  }
}

/// Empty `UndoableChange` that performs no action.
#[derive(Debug, Clone, Copy)]
pub struct NoChange;

impl<S> UndoableChange<S> for NoChange {
  fn play_forward(&self, _: &mut S) {}
  fn play_backward(&self, _: &mut S) {}

  fn undo_summary(&self) -> String {
    "NoChange".to_string()
  }
}
