
use crate::expr::Expr;
use crate::expr::atom::Atom;
use crate::expr::var::Var;
use crate::expr::function::table::FunctionTable;

use thiserror::Error;

/// A `DerivativeEngine` is an engine for recursively evaluating
/// derivatives, in terms of the arguments to a function. Callers
/// outside this module can never construct `DerivativeEngine`
/// instances. They are only used during the recursive descent of the
/// [`differentiate`] function.
///
/// A `DerivativeEngine` remembers the original top-level expression
/// it was created on, to improve error recovery.
#[derive(Debug)]
pub struct DerivativeEngine<'a> {
  target_variable: Var,
  original_expr: Expr,
  function_table: &'a FunctionTable,
}

/// A failure to differentiate a function successfully.
///
/// Constructed by [`DerivativeEngine::error`].
#[derive(Debug, Clone)]
pub struct DifferentiationFailure {
  /// The original top-level expression that we attempted to
  /// differentiate. This is NOT the specific expression where the
  /// error occurred; it's the whole expression that the
  /// [`differentiate`] call was originally made on.
  pub original_expr: Expr,
  /// The reason for the failure.
  pub error: DifferentiationError,
  _priv: (), // Prevent construction outside of this module
}

/// An error during differentiation.
#[derive(Debug, Clone, Error)]
pub enum DifferentiationError {
  #[error("Derivative of function '{0}' is not known")]
  UnknownDerivative(String),
}

impl<'a> DerivativeEngine<'a> {
  /// Attempts to recursively differentiate a sub-expression.
  ///
  /// If you're looking to start a differentiation process, use the
  /// module-level function [`differentiate`]. This method is only for
  /// recursive descent of the differentiation process.
  pub fn differentiate(&self, expr: Expr) -> Result<Expr, DifferentiationFailure> {
    match expr {
      Expr::Call(function, args) => {
        let Some(known_function) = self.function_table.get(&function) else {
          return Err(self.error(DifferentiationError::UnknownDerivative(function)));
        };
        known_function.differentiate(args, self)
      }
      Expr::Atom(Atom::Number(_) | Atom::Complex(_)) => {
        Ok(Expr::zero())
      }
      Expr::Atom(Atom::Var(var)) => {
        if var == self.target_variable {
          Ok(Expr::one())
        } else {
          Ok(Expr::zero())
        }
      }
    }
  }

  /// Helper function which differentiates each argument in turn.
  /// Equivalent to calling [`DerivativeEngine::differentiate`] on
  /// each argument and compiling the results. Short-circuits out if
  /// any errors are encountered.
  pub fn differentiate_each(&self, exprs: Vec<Expr>) -> Result<Vec<Expr>, DifferentiationFailure> {
    exprs.into_iter()
      .map(|expr| self.differentiate(expr))
      .collect()
  }

  /// Produces a [`DifferentiationFailure`] for a failure that
  /// occurred during the process this `DerivativeEngine` was
  /// responsible for.
  pub fn error(&self, reason: DifferentiationError) -> DifferentiationFailure {
    DifferentiationFailure {
      original_expr: self.original_expr.clone(),
      error: reason,
      _priv: (),
    }
  }
}

/// Differentiates the expression in terms of the given variable,
/// given a table of known functions. Returns the derivative or a
/// [`DifferentiationFailure`]. Note that, in the latter case, the
/// original (un-differentiated) expression can be recovered from
/// `failure_object.original_expr`.
pub fn differentiate(function_table: &FunctionTable, expr: Expr, var: Var) -> Result<Expr, DifferentiationFailure> {
  let engine = DerivativeEngine {
    target_variable: var,
    original_expr: expr.clone(),
    function_table,
  };
  engine.differentiate(expr)
}
