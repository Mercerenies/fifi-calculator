
//! Datatypes and prisms for representing expressions as mathematical
//! formulae.
//!
//! Here, we define a "formula" as a binary relation (such as `=` or
//! `<=`) with expressions on both sides.

use crate::util::prism::Prism;
use crate::expr::{Expr, TryFromExprError};

use thiserror::Error;

use std::fmt::{self, Formatter, Display};
use std::convert::TryFrom;
use std::str::FromStr;

/// A formula is defined as an application of a [`FormulaOp`] to two
/// expression arguments.
#[derive(Debug, Clone, PartialEq)]
pub struct Formula {
  pub left: Expr,
  pub op: FormulaOp,
  pub right: Expr,
}

/// A prism which parses an expression as a top-level formula.
#[derive(Clone, Copy, Debug)]
pub struct ExprToFormula;

#[derive(Debug, Clone, Error)]
#[error("Error parsing formula operator")]
pub struct ParseFormulaOpError {
  _priv: (),
}

/// A binary relational operator which takes two expressions and represents a formula.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum FormulaOp {
  Less, LessEq, Eq, Greater, GreaterEq, NotEq,
}

impl Formula {
  pub fn new(left: Expr, op: FormulaOp, right: Expr) -> Formula {
    Formula { left, op, right }
  }
}

impl FormulaOp {
  /// The symbolic name of the operator.
  pub fn name(self) -> &'static str {
    match self {
      FormulaOp::Less => "<",
      FormulaOp::LessEq => "<=",
      FormulaOp::Eq => "=",
      FormulaOp::Greater => ">",
      FormulaOp::GreaterEq => ">=",
      FormulaOp::NotEq => "!=",
    }
  }
}

impl From<Formula> for Expr {
  fn from(formula: Formula) -> Expr {
    Expr::call(formula.op.name(), vec![formula.left, formula.right])
  }
}

impl TryFrom<Expr> for Formula {
  type Error = TryFromExprError;

  fn try_from(expr: Expr) -> Result<Self, Self::Error> {
    const TYPE_NAME: &'static str = "Formula";
    if let Expr::Call(name, args) = expr {
      if args.len() == 2 {
        if let Ok(op) = FormulaOp::from_str(&name) {
          let [left, right] = args.try_into().unwrap(); // unwrap: Just checked the vec length.
          return Ok(Formula { left, op, right });
        }
      }
      Err(TryFromExprError::new(TYPE_NAME, Expr::Call(name, args)))
    } else {
      Err(TryFromExprError::new(TYPE_NAME, expr))
    }
  }
}

impl FromStr for FormulaOp {
  type Err = ParseFormulaOpError;

  fn from_str(s: &str) -> Result<Self, Self::Err> {
    match s {
      "<" => Ok(FormulaOp::Less),
      "<=" => Ok(FormulaOp::LessEq),
      "=" => Ok(FormulaOp::Eq),
      ">" => Ok(FormulaOp::Greater),
      ">=" => Ok(FormulaOp::GreaterEq),
      "!=" => Ok(FormulaOp::NotEq),
      _ => Err(ParseFormulaOpError { _priv: () }),
    }
  }
}

impl Display for FormulaOp {
  fn fmt(&self, f: &mut Formatter) -> fmt::Result {
    f.write_str(self.name())
  }
}

// TODO: We have a lot of prisms that just delegate to TryFrom in some
// capacity. Can we make a single prism type that just does that in
// general?
impl Prism<Expr, Formula> for ExprToFormula {
  fn narrow_type(&self, expr: Expr) -> Result<Formula, Expr> {
    Formula::try_from(expr).map_err(|err| err.original_expr)
  }

  fn widen_type(&self, formula: Formula) -> Expr {
    formula.into()
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn test_formula_into_expr() {
    let formula = Formula::new(Expr::from(10), FormulaOp::Less, Expr::from(20));
    let expr = Expr::from(formula);
    assert_eq!(expr, Expr::call("<", vec![Expr::from(10), Expr::from(20)]));
  }

  #[test]
  fn test_expr_try_into_formula() {
    let expr = Expr::call("<", vec![Expr::from(10), Expr::from(20)]);
    let formula = Formula::try_from(expr).unwrap();
    assert_eq!(formula, Formula::new(Expr::from(10), FormulaOp::Less, Expr::from(20)));
  }

  #[test]
  fn test_expr_try_into_formula_failed_on_arity() {
    let expr = Expr::call("<", vec![Expr::from(10), Expr::from(20), Expr::from(30)]);
    Formula::try_from(expr).unwrap_err();
  }

  #[test]
  fn test_expr_try_into_formula_failed() {
    let expr = Expr::call("foo", vec![Expr::from(10), Expr::from(20)]);
    Formula::try_from(expr).unwrap_err();
  }
}
