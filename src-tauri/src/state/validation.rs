
//! Helpers for validating text from the frontend against various
//! conditions.

use anyhow::Context;

use crate::expr::var::Var;
use crate::expr::number::Number;
use crate::expr::units::parse_composite_unit_expr;
use crate::expr::algebra::term::{Term, TermParser};
use crate::units::parsing::UnitParser;
use crate::units::CompositeUnit;
use crate::units::tagged::Tagged;
use crate::display::language::LanguageMode;

use num::One;
use serde::{Serialize, Deserialize};

/// Types of validations that can be requested of the backend.
#[derive(Debug, Copy, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
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
pub struct ValidationContext<'a, 'b, 'c> {
  pub units_parser: &'a dyn UnitParser<Number>,
  pub term_parser: &'b TermParser,
  pub language_mode: &'c dyn LanguageMode,
}

pub fn validate(validator: Validator, context: &ValidationContext, payload: String) -> anyhow::Result<()> {
  match validator {
    Validator::Variable => validate_var(payload).map(|_| ()),
    Validator::AllUnits => validate_is_all_units(context, &payload).map(|_| ()),
    Validator::HasUnits => validate_has_some_units(context, &payload).map(|_| ()),
  }
}

/// Validates that the given string is a valid variable name.
pub fn validate_var(name: String) -> Result<Var, anyhow::Error> {
  Var::try_from(name).context("Validation failed: invalid variable name")
}

/// Validates that the string is a valid expression which consists
/// only of products, sums, and integer powers of units.
pub fn validate_is_all_units(
  context: &ValidationContext,
  expr: &str,
) -> Result<CompositeUnit<Number>, anyhow::Error> {
  let expr = context.language_mode.parse(expr)?;
  let tagged_expr = parse_composite_unit_expr(context.units_parser, context.term_parser, expr);
  anyhow::ensure!(tagged_expr.value.is_one(), "Could not parse {} as a unit", tagged_expr.value);
  Ok(tagged_expr.unit)
}

/// Validates that the string is a valid expression which contains at
/// least one unit expression in its top-level [term](Term).
pub fn validate_has_some_units(
  context: &ValidationContext,
  expr: &str,
) -> Result<Tagged<Term, Number>, anyhow::Error> {
  let expr = context.language_mode.parse(expr)?;
  let tagged_expr = parse_composite_unit_expr(context.units_parser, context.term_parser, expr);
  anyhow::ensure!(!tagged_expr.unit.is_empty(), "There are no units in the expression {}", tagged_expr.value);
  Ok(tagged_expr)
}

#[cfg(test)]
mod tests {
  use super::*;
  use crate::units::parsing::default_parser;
  use crate::display::language::basic::BasicLanguageMode;
  use crate::expr::basic_parser::ParseError;

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

  #[test]
  fn test_validate_is_all_units() {
    let units_parser = default_parser();
    let term_parser = TermParser::new();
    let language_mode = BasicLanguageMode::from_common_operators();
    let context = ValidationContext {
      units_parser: &units_parser,
      term_parser: &term_parser,
      language_mode: &language_mode,
    };

    validate_is_all_units(&context, "m").unwrap();
    validate_is_all_units(&context, "m^2 * sec^2").unwrap();
    validate_is_all_units(&context, "km / yr").unwrap();
    validate_is_all_units(&context, "degC").unwrap();

    assert_eq!(
      validate_is_all_units(&context, "1").unwrap_err().to_string(),
      "Could not parse 1 as a unit",
    );
    assert_eq!(
      validate_is_all_units(&context, "3 * degC").unwrap_err().to_string(),
      "Could not parse 3 as a unit",
    );
    assert_eq!(
      validate_is_all_units(&context, "degC / 3").unwrap_err().to_string(),
      "Could not parse 1 / 3 as a unit",
    );
    assert_eq!(
      validate_is_all_units(&context, "aaa / 3").unwrap_err().to_string(),
      "Could not parse aaa / 3 as a unit",
    );
    assert_eq!(
      validate_is_all_units(&context, "degC + m").unwrap_err().to_string(),
      "Could not parse +(degC, m) as a unit",
    );
    assert_eq!(
      validate_is_all_units(&context, "EEE").unwrap_err().to_string(),
      "Could not parse EEE as a unit",
    );
    let err = validate_is_all_units(&context, "(").unwrap_err();
    err.downcast::<ParseError>().unwrap();
    let err = validate_is_all_units(&context, "()").unwrap_err();
    err.downcast::<ParseError>().unwrap();
  }

  #[test]
  fn test_validate_has_some_units() {
    let units_parser = default_parser();
    let term_parser = TermParser::new();
    let language_mode = BasicLanguageMode::from_common_operators();
    let context = ValidationContext {
      units_parser: &units_parser,
      term_parser: &term_parser,
      language_mode: &language_mode,
    };

    validate_has_some_units(&context, "m").unwrap();
    validate_has_some_units(&context, "m^2 * sec^2").unwrap();
    validate_has_some_units(&context, "km / yr").unwrap();
    validate_has_some_units(&context, "degC").unwrap();
    validate_has_some_units(&context, "3 * degC").unwrap();
    validate_has_some_units(&context, "degC / 3").unwrap();

    assert_eq!(
      validate_has_some_units(&context, "1").unwrap_err().to_string(),
      "There are no units in the expression 1",
    );
    assert_eq!(
      validate_has_some_units(&context, "aaa / 3").unwrap_err().to_string(),
      "There are no units in the expression aaa / 3",
    );
    assert_eq!(
      validate_has_some_units(&context, "degC + m").unwrap_err().to_string(),
      "There are no units in the expression +(degC, m)",
    );
    assert_eq!(
      validate_has_some_units(&context, "EEE").unwrap_err().to_string(),
      "There are no units in the expression EEE",
    );
    let err = validate_has_some_units(&context, "(").unwrap_err();
    err.downcast::<ParseError>().unwrap();
    let err = validate_has_some_units(&context, "()").unwrap_err();
    err.downcast::<ParseError>().unwrap();
  }
}
