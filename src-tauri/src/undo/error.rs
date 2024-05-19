
use thiserror::Error;

#[derive(Clone, Copy, Debug, Error)]
#[non_exhaustive]
pub enum UndoError {
  #[error("Nothing to undo")]
  NothingToUndo,
  #[error("Nothing to redo")]
  NothingToRedo,
}
