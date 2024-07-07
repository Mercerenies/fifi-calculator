
//! Commands pertaining to unit arithmetic.

use super::functional::UnaryFunctionCommand;
use crate::expr::algebra::term::Term;
use crate::expr::units::try_parse_unit;

/// Unary command which removes units from the targeted stack
/// element(s).
pub fn remove_units_command() -> UnaryFunctionCommand {
  UnaryFunctionCommand::with_context(|arg, ctx| {
    // TODO: Evaluate if this is still valid if we add a mode that
    // treats multiplication as non-commutative.
    let term = Term::parse_expr(arg);
    let term = term.filter_factors(|expr| {
      try_parse_unit(ctx.units_parser, expr.to_owned()).is_err() // TODO: Excessive cloning
    });
    term.into()
  })
}

/// Unary command which keeps only the units from the targeted stack
/// element(s).
pub fn extract_units_command() -> UnaryFunctionCommand {
  UnaryFunctionCommand::with_context(|arg, ctx| {
    // TODO: Evaluate if this is still valid if we add a mode that
    // treats multiplication as non-commutative.
    let term = Term::parse_expr(arg);
    let term = term.filter_factors(|expr| {
      try_parse_unit(ctx.units_parser, expr.to_owned()).is_ok() // TODO: Excessive cloning
    });
    term.into()
  })
}
