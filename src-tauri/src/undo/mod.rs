
//! Undo/redo stack capabilities.

mod change;
mod error;
mod stack;

pub use change::{UndoableChange, NoChange};
pub use error::UndoError;
pub use stack::UndoStack;
