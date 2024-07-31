
// The #[non_exhaustive] attribute applies at the crate-level, and I
// want module-level restrictions, which are far stricter.
#![allow(clippy::manual_non_exhaustive)]

#![warn(clippy::derive_partial_eq_without_eq)]

pub mod command;
pub mod errorlist;
pub mod expr;
pub mod graphics;
pub mod mode;
pub mod parsing;
pub mod runner;
pub mod stack;
pub mod state;
pub mod undo;
pub mod units;
pub mod util;
