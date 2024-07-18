
//! Helpers for treating expressions as scalar quantities tagged with
//! units, such as "inches" or "minutes". Supports composite units
//! (i.e. products, quotients, and integer powers of units).
//!
//! The unit arithmetic portion of this functionality is pulled
//! directly from [`crate::units`], which defines the notions of
//! "unit" and "dimension" in an abstract, generic context. This
//! module itself directly instantiates that functionality for the
//! `Expr` and `Number` types in particular.

mod output;
mod parser;
mod prisms;
mod simplifier;

pub use output::{tagged_into_expr, tagged_into_term, unit_into_term};
pub use parser::{parse_composite_unit_term, parse_composite_unit_expr, try_parse_unit};
pub use prisms::{ParsedCompositeUnit, UnitPrism};
pub use simplifier::UnitSimplifier;
