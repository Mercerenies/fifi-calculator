
//! Helpers for validating text from the frontend against various
//! conditions.

use anyhow::Context;

use crate::expr::var::Var;
use crate::expr::number::Number;
use crate::expr::units::parse_composite_unit_expr;
use crate::expr::algebra::term::{Term, TermParser};
use crate::expr::prisms::{StringToUsize, StringToI64};
use crate::units::parsing::UnitParser;
use crate::units::{Unit, CompositeUnit};
use crate::units::tagged::{Tagged, TemperatureTagged, try_into_basic_temperature_unit};
use crate::mode::display::language::LanguageMode;
use crate::util::radix::{Radix, StringToRadix};
use crate::util::prism::Prism;

use num::One;
use serde::{Serialize, Deserialize};

/// Types of validations that can be requested of the backend.
#[derive(Debug, Copy, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Validator {
  /// Validator that checks whether its input is a valid variable
  /// name. Invokes [`validate_var`].
  Variable,
  /// Validator that checks whether its input is a positive integer
  /// which corresponds to a valid display radix.
  Radix,
  /// Validator which accepts nonnegative integers.
  Usize,
  /// Validator which accepts integers.
  I64,
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
  /// Validator that accepts only a simple temperature unit
  /// expression.
  IsTemperatureUnit,
  /// Validator that accepts an expression (potentially with a scalar
  /// part) whose unit is a 1-dimensional temperature unit.
  HasTemperatureUnit,
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
    Validator::Radix => validate_radix(payload).map(|_| ()),
    Validator::Usize => validate_usize(payload).map(|_| ()),
    Validator::I64 => validate_i64(payload).map(|_| ()),
    Validator::AllUnits => validate_is_all_units(context, &payload).map(|_| ()),
    Validator::HasUnits => validate_has_some_units(context, &payload).map(|_| ()),
    Validator::IsTemperatureUnit => validate_is_temperature_unit(context, &payload).map(|_| ()),
    Validator::HasTemperatureUnit => validate_has_temperature_unit(context, &payload).map(|_| ()),
  }
}

/// Validates that the given string is a valid variable name.
pub fn validate_var(name: String) -> Result<Var, anyhow::Error> {
  Var::try_from(name).context("Validation failed: invalid variable name")
}

/// Validates that the given string is a positive integer which
/// denotes a valid radix value.
pub fn validate_radix(payload: String) -> Result<Radix, anyhow::Error> {
  StringToRadix.narrow_type(payload)
    .map_err(|_| anyhow::anyhow!("Validation failed: invalid radix"))
}

pub fn validate_usize(payload: String) -> Result<usize, anyhow::Error> {
  match StringToUsize.narrow_type(payload) {
    Err(_) => Err(anyhow::anyhow!("Validation failed: invalid integer")),
    Ok(value) => Ok(usize::from(value)),
  }
}

pub fn validate_i64(payload: String) -> Result<i64, anyhow::Error> {
  match StringToI64.narrow_type(payload) {
    Err(_) => Err(anyhow::anyhow!("Validation failed: invalid integer")),
    Ok(value) => Ok(i64::from(value)),
  }
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

pub fn validate_is_temperature_unit(
  context: &ValidationContext,
  expr: &str,
) -> Result<Unit<Number>, anyhow::Error> {
  let composite_unit = validate_is_all_units(context, expr)?;
  let final_unit = try_into_basic_temperature_unit(composite_unit)?;
  Ok(final_unit)
}

pub fn validate_has_temperature_unit(
  context: &ValidationContext,
  expr: &str,
) -> Result<TemperatureTagged<Term, Number>, anyhow::Error> {
  let tagged = validate_has_some_units(context, expr)?;
  let temperature_tagged = TemperatureTagged::try_from(tagged)?;
  Ok(temperature_tagged)
}

#[cfg(test)]
mod tests {
  use super::*;
  use crate::units::parsing::default_parser;
  use crate::mode::display::language::basic::BasicLanguageMode;
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

  #[test]
  fn test_validate_is_temperature_unit() {
    let units_parser = default_parser();
    let term_parser = TermParser::new();
    let language_mode = BasicLanguageMode::from_common_operators();
    let context = ValidationContext {
      units_parser: &units_parser,
      term_parser: &term_parser,
      language_mode: &language_mode,
    };

    validate_is_temperature_unit(&context, "degC").unwrap();
    validate_is_temperature_unit(&context, "K").unwrap();
    validate_is_temperature_unit(&context, "dF").unwrap();
    validate_is_temperature_unit(&context, "uK").unwrap(); // micro-Kelvins ;)

    assert_eq!(
      validate_is_temperature_unit(&context, "1").unwrap_err().to_string(),
      "Could not parse 1 as a unit",
    );
    assert_eq!(
      validate_is_temperature_unit(&context, "3 * degC").unwrap_err().to_string(),
      "Could not parse 3 as a unit",
    );
    assert_eq!(
      validate_is_temperature_unit(&context, "degC / 3").unwrap_err().to_string(),
      "Could not parse 1 / 3 as a unit",
    );
    assert_eq!(
      validate_is_temperature_unit(&context, "aaa / 3").unwrap_err().to_string(),
      "Could not parse aaa / 3 as a unit",
    );
    assert_eq!(
      validate_is_temperature_unit(&context, "degC + m").unwrap_err().to_string(),
      "Could not parse +(degC, m) as a unit",
    );
    assert_eq!(
      validate_is_temperature_unit(&context, "EEE").unwrap_err().to_string(),
      "Could not parse EEE as a unit",
    );
    assert_eq!(
      validate_is_temperature_unit(&context, "km").unwrap_err().to_string(),
      "Expected temperature unit",
    );
    assert_eq!(
      validate_is_temperature_unit(&context, "degC * rad").unwrap_err().to_string(),
      "Expected temperature unit",
    );
    let err = validate_is_temperature_unit(&context, "(").unwrap_err();
    err.downcast::<ParseError>().unwrap();
    let err = validate_is_temperature_unit(&context, "()").unwrap_err();
    err.downcast::<ParseError>().unwrap();
  }

  #[test]
  fn test_validate_has_temperature_unit() {
    let units_parser = default_parser();
    let term_parser = TermParser::new();
    let language_mode = BasicLanguageMode::from_common_operators();
    let context = ValidationContext {
      units_parser: &units_parser,
      term_parser: &term_parser,
      language_mode: &language_mode,
    };

    validate_has_temperature_unit(&context, "degC").unwrap();
    validate_has_temperature_unit(&context, "K").unwrap();
    validate_has_temperature_unit(&context, "dF").unwrap();
    validate_has_temperature_unit(&context, "uK").unwrap(); // micro-Kelvins ;)
    validate_has_temperature_unit(&context, "3 * dF").unwrap();
    validate_has_temperature_unit(&context, "degC / 10").unwrap();

    assert_eq!(
      validate_has_temperature_unit(&context, "1").unwrap_err().to_string(),
      "There are no units in the expression 1",
    );
    assert_eq!(
      validate_has_temperature_unit(&context, "aaa / 3").unwrap_err().to_string(),
      "There are no units in the expression aaa / 3",
    );
    assert_eq!(
      validate_has_temperature_unit(&context, "km").unwrap_err().to_string(),
      "Expected temperature unit",
    );
    assert_eq!(
      validate_has_temperature_unit(&context, "3 * km").unwrap_err().to_string(),
      "Expected temperature unit",
    );
    let err = validate_has_temperature_unit(&context, "(").unwrap_err();
    err.downcast::<ParseError>().unwrap();
    let err = validate_has_temperature_unit(&context, "()").unwrap_err();
    err.downcast::<ParseError>().unwrap();
  }
}
