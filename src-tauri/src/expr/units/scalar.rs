
use crate::expr::Expr;
use crate::expr::var::Var;
use crate::expr::number::Number;
use crate::expr::algebra::term::Term;
use crate::expr::prisms;
use crate::units::unit::{UnitWithPower, CompositeUnit};
use crate::units::parsing::UnitParser;
use crate::units::tagged::Tagged;
use crate::util::partition_mapped;
use crate::util::prism::{Prism, PrismExt};

use either::Either;

#[derive(Debug)]
struct PowerExprPrism;

pub fn parse_composite_unit_term<P>(unit_parser: &P, term: Term) -> Tagged<Term, Number>
where P: UnitParser<Number> {
  let (numerator, denominator) = term.into_parts();
  let (term_numerator, unit_numerator): (Vec<_>, Vec<_>) =
    partition_mapped(numerator, |expr| try_parse_unit(unit_parser, expr).into());
  let (term_denominator, unit_denominator): (Vec<_>, Vec<_>) =
    partition_mapped(denominator, |expr| try_parse_unit(unit_parser, expr).into());

  let unit = CompositeUnit::new(unit_numerator) / CompositeUnit::new(unit_denominator);
  Tagged {
    value: Term::new(term_numerator, term_denominator),
    unit,
  }
}

pub fn parse_composite_unit_expr<P>(unit_parser: &P, expr: Expr) -> Tagged<Term, Number>
where P: UnitParser<Number> {
  let term = Term::parse_expr(expr);
  parse_composite_unit_term(unit_parser, term)
}

fn unit_like_expr_prism() -> impl Prism<Expr, Either<(Var, i64), Var>> {
  PowerExprPrism.or(prisms::expr_to_var())
}

fn try_parse_unit<P>(unit_parser: &P, expr: Expr) -> Result<UnitWithPower<Number>, Expr>
where P: UnitParser<Number> {
  let prism = unit_like_expr_prism();
  prism.narrow_type(expr).and_then(|value| {
    let (var, n) = match value {
      Either::Left((var, n)) => (var, n),
      Either::Right(var) => (var, 1),
    };
    match unit_parser.parse_unit(var.as_str()) {
      Ok(unit) => {
        Ok(UnitWithPower { unit, exponent: n })
      }
      Err(_) => {
        if n == 1 {
          Err(prism.widen_type(Either::Right(var)))
        } else {
          Err(prism.widen_type(Either::Left((var, n))))
        }
      }
    }
  })
}

impl Prism<Expr, (Var, i64)> for PowerExprPrism {
  fn narrow_type(&self, expr: Expr) -> Result<(Var, i64), Expr> {
    if let Expr::Call(function_name, args) = expr {
      if function_name == "^" && args.len() == 2 {
        let [var, n] = args.try_into().unwrap();
        prisms::expr_to_var().and(prisms::expr_to_i64())
          .narrow_type((var, n))
          .map_err(|(var, n)| Expr::call("^", vec![var, n]))
      } else {
        Err(Expr::Call(function_name, args))
      }
    } else {
      Err(expr)
    }
  }

  fn widen_type(&self, (var, n): (Var, i64)) -> Expr {
    Expr::call("^", vec![var.into(), n.into()])
  }
}

#[cfg(test)]
mod tests {
  use super::*;
  use crate::units::unit::Unit;
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
    TableBasedParser { table }
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
    assert_eq!(value, Term::new(
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
