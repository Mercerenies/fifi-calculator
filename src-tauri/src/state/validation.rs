
//! Helpers for validating text from the frontend against various
//! conditions.

use anyhow::Context;

use crate::expr::var::Var;
use crate::expr::number::Number;
use crate::expr::units::parse_composite_unit_expr;
use crate::expr::algebra::term::Term;
use crate::units::parsing::UnitParser;
use crate::units::unit::CompositeUnit;
use crate::units::tagged::Tagged;
use crate::display::language::LanguageMode;

use num::One;
use serde::{Serialize, Deserialize};

/// Types of validations that can be requested of the backend.
#[derive(Debug, Copy, Clone, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Validator {
  /// Validator that checks whether its input is a valid variable
  /// name. Invokes [`validate_var`].
  Variable,
  /// Validator that only accepts expressions which can be fully
  /// parsed as a unit expression. Delegates to
  /// [`validate_is_all_units`].
  ///
  /// This is a strictly stronger constraint than
  /// `Validator::HasUnits`, for all non-empty terms (That is, for all
  /// expressions except extremely contrived ones like `*() / *()`).
  AllUnits,
  /// Validator that accepts expressions which contain at least one
  /// parse-able unit, even if the entire expression is not a unit
  /// expression. Delegates to [`validate_has_some_units`].
  HasUnits,
}

#[derive(Clone)]
pub struct ValidationContext<'a, 'b> {
  pub units_parser: &'a dyn UnitParser<Number>,
  pub language_mode: &'b dyn LanguageMode,
}

pub fn validate(validator: Validator, context: &ValidationContext, payload: String) -> anyhow::Result<()> {
  match validator {
    Validator::Variable => validate_var(payload).map(|_| ()),
    Validator::AllUnits => validate_is_all_units(context, payload).map(|_| ()),
    Validator::HasUnits => validate_has_some_units(context, payload).map(|_| ()),
  }
}

/// Validates that the given string is a valid variable name.
pub fn validate_var(name: String) -> Result<Var, anyhow::Error> {
  Var::try_from(name).context("Validation failed: invalid variable name")
}

pub fn validate_is_all_units(
  context: &ValidationContext,
  expr: String,
) -> Result<CompositeUnit<Number>, anyhow::Error> {
  let expr = context.language_mode.parse(&expr)?;
  let tagged_expr = parse_composite_unit_expr(context.units_parser, expr);
  anyhow::ensure!(tagged_expr.value.is_one(), "Could not parse {} as a unit", tagged_expr.value);
  Ok(tagged_expr.unit)
}

pub fn validate_has_some_units(
  context: &ValidationContext,
  expr: String,
) -> Result<Tagged<Term, Number>, anyhow::Error> {
  let expr = context.language_mode.parse(&expr)?;
  let tagged_expr = parse_composite_unit_expr(context.units_parser, expr);
  anyhow::ensure!(!tagged_expr.unit.is_empty(), "There are no units in the expression {}", tagged_expr.value);
  Ok(tagged_expr)
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn test_validate_var_valid() {
    validate_var("abc".to_owned()).unwrap();
    validate_var("abc'".to_owned()).unwrap();
    validate_var("abc123".to_owned()).unwrap();
  }

  #[test]
  fn test_validate_var_invalid() {
    validate_var("3".to_owned()).unwrap_err();
    validate_var("''".to_owned()).unwrap_err();
    validate_var("a_b".to_owned()).unwrap_err();
  }
}

///// tests for the rest of these
