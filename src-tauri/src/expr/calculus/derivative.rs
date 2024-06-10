
use crate::expr::Expr;
use crate::expr::atom::Atom;
use crate::expr::var::Var;
use crate::expr::function::table::FunctionTable;

use thiserror::Error;

#[derive(Debug)]
pub struct DerivativeEngine<'a> {
  target_variable: Var,
  original_expr: Expr,
  function_table: &'a FunctionTable,
}

#[derive(Debug, Clone)]
pub struct DifferentiationFailure {
  pub original_expr: Expr,
  pub error: DifferentiationError,
  _priv: (), // Prevent construction outside of this module
}

#[derive(Debug, Clone, Error)]
pub enum DifferentiationError {
  #[error("Derivative of function '{0}' is not known")]
  UnknownDerivative(String),
}

impl<'a> DerivativeEngine<'a> {
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

  pub fn error(&self, reason: DifferentiationError) -> DifferentiationFailure {
    DifferentiationFailure {
      original_expr: self.original_expr.clone(),
      error: reason,
      _priv: (),
    }
  }
}

pub fn differentiate(function_table: &FunctionTable, expr: Expr, var: Var) -> Result<Expr, DifferentiationFailure> {
  let engine = DerivativeEngine {
    target_variable: var,
    original_expr: expr.clone(),
    function_table,
  };
  engine.differentiate(expr)
}
