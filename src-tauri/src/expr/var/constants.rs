
//! Well-known mathematical constants that are useful to the user.

use crate::expr::Expr;
use crate::expr::number::ComplexNumber;
use super::Var;
use super::table::VarTable;

use thiserror::Error;
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

#[derive(Error, Debug, Clone)]
#[error("variable name '{name}' is reserved")]
pub struct NameIsReservedError {
  name: Var,
}

/// The Euler-Mascheroni constant. The std::f64::consts constant is
/// nightly-only.
const GAMMA: f64 = 0.577_215_664_901_532_9_f64;

/// The golden ratio. The std::f64::consts constant is
/// nightly-only.
const PHI: f64 = 1.618_033_988_749_895_f64;

/// Binds the well-known constants in the given variable table.
pub fn bind_constants(table: &mut VarTable<Expr>) {
  table.insert(Var::new("pi").unwrap(), Expr::from(PI));
  table.insert(Var::new("gamma").unwrap(), Expr::from(GAMMA));
  table.insert(Var::new("e").unwrap(), Expr::from(E));
  table.insert(Var::new("i").unwrap(), Expr::from(ComplexNumber::ii()));
  table.insert(Var::new("phi").unwrap(), Expr::from(PHI));
}

/// If the variable name is a reserved name, returns an appropriate
/// error. Otherwise, returns `Ok`.
pub fn validate_non_reserved_var_name(name: &Var) -> Result<(), NameIsReservedError> {
  if RESERVED_NAMES.contains(name) {
    Err(NameIsReservedError { name: name.clone() })
  } else {
    Ok(())
  }
}
