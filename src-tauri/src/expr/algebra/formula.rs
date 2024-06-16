
//! Datatypes and prisms for representing expressions as mathematical
//! formulae.
//!
//! Here, we define a "formula" as a binary relation (such as `=` or
//! `<=`) with expressions on both sides.

use crate::expr::{Expr, TryFromExprError};
use crate::util::prism::ErrorWithPayload;

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

/// An `Equation` is a [`Formula`] whose operator is
/// [`FormulaOp::Eq`]. This type is provided as the target for a
/// prism, since many calculator operations require an equation
/// specifically.
#[derive(Debug, Clone, PartialEq)]
pub struct Equation {
  pub left: Expr,
  pub right: Expr,
}

#[derive(Debug, Clone, Error)]
#[error("Error parsing formula operator")]
pub struct ParseFormulaOpError {
  _priv: (),
}

#[derive(Debug, Clone, Error)]
#[error("Expecting equation")]
pub struct FormulaToEquationError {
  formula: Formula,
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

impl Equation {
  pub fn new(left: Expr, right: Expr) -> Equation {
    Equation { left, right }
  }

  pub fn equals_zero(left: Expr) -> Equation {
    Equation::new(left, Expr::zero())
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

impl From<Equation> for Formula {
  fn from(equation: Equation) -> Formula {
    Formula { left: equation.left, op: FormulaOp::Eq, right: equation.right }
  }
}

impl TryFrom<Formula> for Equation {
  type Error = FormulaToEquationError;

  fn try_from(formula: Formula) -> Result<Self, Self::Error> {
    if formula.op == FormulaOp::Eq {
      Ok(Equation { left: formula.left, right: formula.right })
    } else {
      Err(FormulaToEquationError { formula })
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

impl ErrorWithPayload<Formula> for FormulaToEquationError {
  fn recover_payload(self) -> Formula {
    self.formula
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
