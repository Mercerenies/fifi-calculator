
use crate::expr::Expr;
use crate::expr::algebra::term::Term;
use crate::units::tagged::Tagged;
use crate::units::unit::{Unit, CompositeUnit};

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

pub fn unit_into_term<S>(composite_unit: CompositeUnit<S>) -> Result<Term, UnitIntoTermError> {
  let mut numerator = Vec::new();
  let mut denominator = Vec::new();
  for unit in composite_unit.into_inner() {
    let var = parse_var(&unit.unit)?;
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
  Ok(Term::new(numerator, denominator))
}

fn parse_var<S>(unit: &Unit<S>) -> Result<Expr, UnitIntoTermError> {
  Expr::var(unit.name())
    .ok_or_else(|| UnitIntoTermError::InvalidVariableName(unit.name().to_owned()))
}

#[cfg(test)]
mod tests {
  use super::*;
  use crate::units::dimension::BaseDimension;
  use crate::units::unit::{UnitWithPower};

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
    assert_eq!(
      unit_into_term(composite_unit),
      Ok(Term::new(vec![var("cd"), pow(var("km"), 2)], vec![var("mol"), pow(var("sec"), 3)])),
    );
  }
}
