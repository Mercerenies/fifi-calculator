
//! Defines simplifiers which use the [`Term`] abstraction to perform
//! simplification.
//!
//! See [`TermPartialSplitter`] and [`FactorSorter`] for more details.

mod splitter;
mod factor;

pub use splitter::TermPartialSplitter;
pub use factor::FactorSorter;
