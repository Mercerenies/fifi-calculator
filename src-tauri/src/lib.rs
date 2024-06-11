
// The #[non_exhaustive] attribute applies at the crate-level, and I
// want module-level restrictions, which are far stricter.
#![allow(clippy::manual_non_exhaustive)]

pub mod command;
pub mod display;
pub mod errorlist;
pub mod expr;
pub mod parsing;
pub mod stack;
pub mod state;
pub mod undo;
pub mod util;
