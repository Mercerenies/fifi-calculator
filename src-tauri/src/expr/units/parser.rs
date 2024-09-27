
use crate::expr::Expr;
use crate::expr::atom::Atom;
use crate::expr::algebra::term::Term;
use crate::expr::algebra::factor::Factor;
use crate::expr::prisms;
use crate::units::{UnitWithPower, CompositeUnit};
use crate::units::parsing::UnitParser;
use crate::units::tagged::Tagged;
use crate::util::partition_mapped;
use crate::util::prism::Prism;

use either::Either;
use thiserror::Error;

#[derive(Clone, Debug, Error)]
#[non_exhaustive]
#[error("Could not parse unit")]
pub struct TryParseUnitError {}

pub fn parse_composite_unit_term<P, T>(unit_parser: &P, term: Term) -> Tagged<Term, T>
where P: UnitParser<T> + ?Sized {
  let discriminate = |factor: Factor| -> Either<Factor, UnitWithPower<T>> {
    match try_parse_unit(unit_parser, &factor) {
      Ok(unit) => Either::Right(unit),
      Err(_) => Either::Left(factor),
    }
  };

  let (numerator, denominator) = term.into_parts();
  let (term_numerator, unit_numerator): (Vec<_>, Vec<_>) = partition_mapped(numerator, discriminate);
  let (term_denominator, unit_denominator): (Vec<_>, Vec<_>) = partition_mapped(denominator, discriminate);

  let unit = CompositeUnit::new(unit_numerator) / CompositeUnit::new(unit_denominator);
  Tagged {
    value: Term::from_parts(term_numerator, term_denominator),
    unit,
  }
}

pub fn parse_composite_unit_expr<P, T>(unit_parser: &P, expr: Expr) -> Tagged<Term, T>
where P: UnitParser<T> + ?Sized {
  let term = Term::parse(expr);
  parse_composite_unit_term(unit_parser, term)
}

pub fn try_parse_unit<P, T>(unit_parser: &P, factor: &Factor) -> Result<UnitWithPower<T>, TryParseUnitError>
where P: UnitParser<T> + ?Sized {
  let Expr::Atom(Atom::Var(base)) = factor.base() else {
    return Err(TryParseUnitError {});
  };
  let Ok(exp) = prisms::expr_to_i64().narrow_type(factor.exponent_or_one()) else {
    return Err(TryParseUnitError {});
  };
  let Ok(unit) = unit_parser.parse_unit(base.as_str()) else {
    return Err(TryParseUnitError {});
  };
  Ok(UnitWithPower { unit, exponent: exp })
}

#[cfg(test)]
mod tests {
  use super::*;
  use crate::units::Unit;
  use crate::expr::number::Number;
  use crate::units::dimension::BaseDimension;
  use crate::units::parsing::TableBasedParser;

  use std::collections::HashMap;

  fn var(n: &str) -> Expr {
    Expr::var(n).unwrap()
  }

  fn sample_table() -> TableBasedParser<Number> {
    let mut table = HashMap::new();
    table.insert("m".to_owned(), Unit::new("m", BaseDimension::Length, Number::from(1)));
    table.insert("s".to_owned(), Unit::new("s", BaseDimension::Time, Number::from(1)));
    table.insert("degC".to_owned(), Unit::new("degC", BaseDimension::Temperature, Number::from(1)));
    TableBasedParser::new(table, |_| panic!("Should not be called"))
  }

  #[test]
  fn test_parse_composite_unit() {
    let table_parser = sample_table();
    let expr = Expr::call("/", vec![
      Expr::call("*", vec![
        var("m"),
        Expr::call("*", vec![Expr::from(100), var("s"), var("xxx")]),
        var("degC"),
        var("degC"),
      ]),
      Expr::call("*", vec![
        Expr::call("^", vec![var("s"), Expr::from(2)]),
        Expr::call("^", vec![var("yyy"), Expr::from(2)]),
        Expr::from(200),
        Expr::call("^", vec![var("degC"), Expr::from(2)]),
      ]),
    ]);
    let Tagged { value, unit } = parse_composite_unit_expr(&table_parser, expr);
    assert_eq!(value, Term::from_parts(
      vec![Expr::from(100), var("xxx")],
      vec![Expr::call("^", vec![var("yyy"), Expr::from(2)]), Expr::from(200)],
    ));
    assert_eq!(unit, CompositeUnit::new([
      // Note: No degC term since they cancelled off.
      UnitWithPower {
        unit: Unit::new("m", BaseDimension::Length, Number::from(1)),
        exponent: 1,
      },
      UnitWithPower {
        unit: Unit::new("s", BaseDimension::Time, Number::from(1)),
        exponent: -1,
      },
    ]));
  }
}
