
//! Helpers for manipulating expressions algebraically.

pub mod formula;
pub mod newton;

use crate::errorlist::ErrorList;
use crate::util::prism::Prism;
use crate::expr::Expr;
use crate::expr::number::{ComplexLike, Number};
use crate::expr::var::Var;
use crate::expr::simplifier::{Simplifier, SimplifierContext};
use crate::expr::prisms::ExprToComplex;

use thiserror::Error;

/// An expression, being treated as a function of one argument,
/// subject to a particular simplifier.
pub struct ExprFunction<'a> {
  expr: Expr,
  var: Var,
  simplifier: &'a dyn Simplifier,
}

/// An error during function evaluation.
#[derive(Debug, Clone, Error)]
#[error("Failed to evaluate function, expecting {expected}, got {evaluated_value}")]
pub struct FunctionEvalError {
  pub evaluated_value: Expr,
  pub function: Expr,
  pub expected: &'static str,
  _priv: (),
}

impl<'a> ExprFunction<'a> {
  /// Create a new expression function.
  pub fn new(expr: Expr, var: Var, simplifier: &'a dyn Simplifier) -> ExprFunction<'a> {
    ExprFunction {
      expr,
      var,
      simplifier,
    }
  }

  /// Evaluates the function at the given position, expecting a
  /// numerical result.
  pub fn eval_at(&self, value: ComplexLike) -> Result<ComplexLike, FunctionEvalError> {
    let evaluated_value = self.expr.clone().substitute_var(self.var.clone(), value.into());
    let evaluated_value = self.simplify_expr(evaluated_value);
    ExprToComplex.narrow_type(evaluated_value)
      .map_err(|evaluated_value| FunctionEvalError {
        evaluated_value,
        function: self.expr.clone(),
        expected: "numerical literal",
        _priv: (),
      })
  }

  /// Evaluates the function at the given position, expecting a real
  /// numerical result.
  pub fn eval_at_real(&self, value: Number) -> Result<Number, FunctionEvalError> {
    match self.eval_at(ComplexLike::Real(value))? {
      ComplexLike::Real(r) => Ok(r),
      ComplexLike::Complex(z) => {
        Err(FunctionEvalError {
          evaluated_value: z.into(),
          function: self.expr.clone(),
          expected: "real number",
          _priv: (),
        })
      }
    }
  }

  fn simplify_expr(&self, expr: Expr) -> Expr {
    // Note: When we simplify expressions for the purpose of
    // ExprFunction, we ignore any errors that arise from the
    // simplifier.
    let mut errors = ErrorList::new();
    let mut context = SimplifierContext { base_simplifier: self.simplifier, errors: &mut errors };
    self.simplifier.simplify_expr(expr, &mut context)
  }
}
