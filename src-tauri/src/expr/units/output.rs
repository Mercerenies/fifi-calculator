
use crate::expr::Expr;
use crate::expr::algebra::term::Term;
use crate::units::tagged::Tagged;
use crate::units::{Unit, CompositeUnit};

use thiserror::Error;

#[derive(Clone, Debug, Error, PartialEq, Eq)]
#[non_exhaustive]
pub enum UnitIntoTermError {
  #[error("Unit {0} could not be represented as a variable in the expression language")]
  InvalidVariableName(String),
}

pub fn tagged_into_expr<S>(tagged: Tagged<Term, S>) -> Result<Expr, UnitIntoTermError> {
  let term = tagged_into_term(tagged)?;
  Ok(term.into())
}

pub fn tagged_into_term<S>(tagged: Tagged<Term, S>) -> Result<Term, UnitIntoTermError> {
  Ok(tagged.value * unit_into_term(tagged.unit)?)
}

/// Converts a composite unit into a term, producing an error if any
/// of the constituent units are not valid variable names.
pub fn unit_into_term<S>(composite_unit: CompositeUnit<S>) -> Result<Term, UnitIntoTermError> {
  unit_into_term_impl(composite_unit, false)
}

pub fn tagged_into_expr_lossy<S>(tagged: Tagged<Term, S>) -> Expr {
  tagged_into_term_lossy(tagged).into()
}

pub fn tagged_into_term_lossy<S>(tagged: Tagged<Term, S>) -> Term {
  tagged.value * unit_into_term_lossy(tagged.unit)
}

/// Converts a composite unit into a term, skipping any constituent
/// units which are not valid variable names.
pub fn unit_into_term_lossy<S>(composite_unit: CompositeUnit<S>) -> Term {
  // unwrap: unit_into_term_impl never errs if `lossy == true`
  unit_into_term_impl(composite_unit, true).unwrap()
}

fn unit_into_term_impl<S>(composite_unit: CompositeUnit<S>, lossy: bool) -> Result<Term, UnitIntoTermError> {
  let mut numerator = Vec::new();
  let mut denominator = Vec::new();
  for unit in composite_unit.into_inner() {
    let var = match parse_var(&unit.unit) {
      Ok(var) => var,
      Err(err) => {
        if lossy {
          continue;
        } else {
          return Err(err);
        }
      }
    };
    match unit.exponent {
      0 => {
        // Do not include this unit in the result.
      }
      1 => {
        numerator.push(var);
      }
      -1 => {
        denominator.push(var);
      }
      x if x > 0 => {
        numerator.push(
          Expr::call("^", vec![var, Expr::from(x)]),
        );
      }
      x => {
        denominator.push(
          Expr::call("^", vec![var, Expr::from(-x)]),
        );
      }
    }
  }
  Ok(Term::from_parts(numerator, denominator))
}

fn parse_var<S>(unit: &Unit<S>) -> Result<Expr, UnitIntoTermError> {
  Expr::var(unit.name())
    .ok_or_else(|| UnitIntoTermError::InvalidVariableName(unit.name().to_owned()))
}

#[cfg(test)]
mod tests {
  use super::*;
  use crate::units::dimension::BaseDimension;
  use crate::units::UnitWithPower;

  fn var(x: &str) -> Expr {
    Expr::var(x).unwrap()
  }

  fn pow(expr: Expr, exponent: i64) -> Expr {
    Expr::call("^", vec![expr, Expr::from(exponent)])
  }

  #[test]
  fn test_parse_var() {
    assert_eq!(
      parse_var(&Unit::new("km", BaseDimension::Length, 1_000)),
      Ok(Expr::var("km").unwrap()),
    );
    assert_eq!(
      parse_var(&Unit::new("invalid variable name", BaseDimension::Length, 1_000)),
      Err(UnitIntoTermError::InvalidVariableName("invalid variable name".to_owned())),
    );
  }

  #[test]
  fn test_unit_into_term() {
    let composite_unit = CompositeUnit::new(vec![
      UnitWithPower { unit: Unit::new("km", BaseDimension::Length, 1_000), exponent: 2 },
      UnitWithPower { unit: Unit::new("sec", BaseDimension::Time, 1), exponent: -3 },
      UnitWithPower { unit: Unit::new("cd", BaseDimension::LuminousIntensity, 1), exponent: 1 },
      UnitWithPower { unit: Unit::new("mol", BaseDimension::AmountOfSubstance, 1), exponent: -1 },
    ]);
    let expected_output = Term::from_parts(
      [var("cd"), pow(var("km"), 2)],
      [var("mol"), pow(var("sec"), 3)],
    );
    assert_eq!(unit_into_term(composite_unit).unwrap(), expected_output);
  }

  #[test]
  fn test_unit_into_term_failure() {
    let composite_unit = CompositeUnit::new(vec![
      UnitWithPower { unit: Unit::new("a b", BaseDimension::Length, 1_000), exponent: 2 },
    ]);
    assert_eq!(
      unit_into_term(composite_unit),
      Err(UnitIntoTermError::InvalidVariableName("a b".to_owned())),
    );
  }

  #[test]
  fn test_unit_into_term_lossy() {
    let composite_unit = CompositeUnit::new(vec![
      UnitWithPower { unit: Unit::new("km", BaseDimension::Length, 1_000), exponent: 2 },
      UnitWithPower { unit: Unit::new("sec", BaseDimension::Time, 1), exponent: -3 },
      UnitWithPower { unit: Unit::new("invalid unit name", BaseDimension::LuminousIntensity, 1), exponent: 1 },
      UnitWithPower { unit: Unit::new("mol", BaseDimension::AmountOfSubstance, 1), exponent: -1 },
    ]);
    assert_eq!(
      unit_into_term_lossy(composite_unit),
      Term::from_parts(
        [pow(var("km"), 2)],
        [var("mol"), pow(var("sec"), 3)],
      ),
    );
  }

  #[test]
  fn test_tagged_into_term() {
    let term = Term::from_parts([Expr::from(100)], [Expr::from(101)]);
    let composite_unit = CompositeUnit::new(vec![
      UnitWithPower { unit: Unit::new("km", BaseDimension::Length, 1_000), exponent: 2 },
      UnitWithPower { unit: Unit::new("sec", BaseDimension::Time, 1), exponent: -3 },
      UnitWithPower { unit: Unit::new("cd", BaseDimension::LuminousIntensity, 1), exponent: 1 },
      UnitWithPower { unit: Unit::new("mol", BaseDimension::AmountOfSubstance, 1), exponent: -1 },
    ]);
    let tagged_term = Tagged::new(term, composite_unit);
    assert_eq!(
      tagged_into_term(tagged_term),
      Ok(Term::from_parts(
        [Expr::from(100), var("cd"), pow(var("km"), 2)],
        [Expr::from(101), var("mol"), pow(var("sec"), 3)],
      )),
    );
  }

  #[test]
  fn test_tagged_into_expr() {
    let term = Term::from_parts([Expr::from(100)], [Expr::from(101)]);
    let composite_unit = CompositeUnit::new(vec![
      UnitWithPower { unit: Unit::new("km", BaseDimension::Length, 1_000), exponent: 2 },
      UnitWithPower { unit: Unit::new("sec", BaseDimension::Time, 1), exponent: -3 },
      UnitWithPower { unit: Unit::new("cd", BaseDimension::LuminousIntensity, 1), exponent: 1 },
      UnitWithPower { unit: Unit::new("mol", BaseDimension::AmountOfSubstance, 1), exponent: -1 },
    ]);
    let tagged_term = Tagged::new(term, composite_unit);
    assert_eq!(
      tagged_into_expr(tagged_term),
      Ok(Expr::call("/", vec![
        Expr::call("*", vec![Expr::from(100), var("cd"), pow(var("km"), 2)]),
        Expr::call("*", vec![Expr::from(101), var("mol"), pow(var("sec"), 3)]),
      ])),
    );
  }
}
