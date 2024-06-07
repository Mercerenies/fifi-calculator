
//! Well-known mathematical constants that are useful to the user.

use crate::expr::Expr;
use crate::expr::number::ComplexNumber;
use super::Var;
use super::table::VarTable;

use once_cell::sync::Lazy;

use std::collections::HashSet;
use std::f64::consts::{PI, E};

/// Names reserved for use by our engine. The user should not be
/// permitted to use these names as their own variables.
pub static RESERVED_NAMES: Lazy<HashSet<Var>> = Lazy::new(|| {
  vec![
    // Ordinary constants
    "pi", "gamma", "e", "i", "phi",
    // Symbolic names used by our algebra system
    "inf", "uinf", "nan",
  ].into_iter().map(|s| Var::new(s).unwrap()).collect()
});

/// The Euler-Mascheroni constant. The std::f64::consts constant is
/// nightly-only.
const GAMMA: f64 = 0.57721566490153286060651209008240243104215933593992_f64;

/// The golden ratio. The std::f64::consts constant is
/// nightly-only.
const PHI: f64 = 1.61803398874989484820458683436563811772030917980576_f64;

/// Binds the well-known constants in the given variable table.
pub fn bind_constants(table: &mut VarTable<Expr>) {
  table.insert(Var::new("pi").unwrap(), Expr::from(PI));
  table.insert(Var::new("gamma").unwrap(), Expr::from(GAMMA));
  table.insert(Var::new("e").unwrap(), Expr::from(E));
  table.insert(Var::new("i").unwrap(), Expr::from(ComplexNumber::ii()));
  table.insert(Var::new("phi").unwrap(), Expr::from(PHI));
}
