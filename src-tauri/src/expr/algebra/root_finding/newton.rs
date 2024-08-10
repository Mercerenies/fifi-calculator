
//! Implementation of the Newton-Raphson method for finding roots to
//! differentiable functions.
//!
//! See <https://en.wikipedia.org/wiki/Newton%27s_method>.

use super::FoundRoot;
use crate::expr::Expr;
use crate::expr::algebra::{ExprFunction, FunctionEvalError};
use crate::expr::simplifier::Simplifier;
use crate::expr::var::Var;
use crate::expr::function::table::FunctionTable;
use crate::expr::number::ComplexLike;
use crate::expr::calculus::{differentiate, DifferentiationError, DifferentiationFailure};

use thiserror::Error;
use num::Zero;

#[derive(Debug)]
pub struct NewtonRaphsonMethod {
  epsilon_squared: f64,
  max_iterations: usize,
}

pub struct NewtonRaphsonFunction<'a> {
  function: ExprFunction<'a>,
  derivative: ExprFunction<'a>,
}

#[derive(Debug, Clone, Error)]
#[non_exhaustive]
pub enum NewtonRaphsonError {
  #[error("{0}")]
  DifferentiationError(#[from] DifferentiationError),
  #[error("{0}")]
  FunctionEvalError(#[from] FunctionEvalError),
  #[error("Failed to converge after {iterations} iterations")]
  FailedToConverge { iterations: usize },
  #[error("Division by zero during Newton-Raphson convergence")]
  DivisionByZero,
}

impl NewtonRaphsonMethod {
  pub const DEFAULT_EPSILON: f64 = 1e-5;
  pub const DEFAULT_MAX_ITERATIONS: usize = 1000;

  pub fn new(epsilon: f64, max_iterations: usize) -> Self {
    Self {
      epsilon_squared: epsilon * epsilon,
      max_iterations,
    }
  }

  pub fn find_root(
    &self,
    function: NewtonRaphsonFunction,
    initial_guess: ComplexLike,
  ) -> Result<FoundRoot<ComplexLike>, NewtonRaphsonError> {
    let mut current_value = initial_guess;
    for _ in 0..self.max_iterations {
      // First, check if we're already within epsilon of a root.
      let f = function.eval_at(current_value.clone())?;
      if f.abs_sqr().to_f64().unwrap_or(f64::INFINITY) < self.epsilon_squared {
        let final_epsilon = f.abs().to_f64().unwrap_or(f64::NAN);
        return Ok(FoundRoot { value: current_value, final_epsilon });
      }
      // Else, take the derivative and update the guess.
      let f_prime = function.eval_deriv_at(current_value.clone())?;
      if f_prime.is_zero() {
        return Err(NewtonRaphsonError::DivisionByZero);
      }
      current_value = current_value - f / f_prime;
    }
    Err(NewtonRaphsonError::FailedToConverge { iterations: self.max_iterations })
  }
}

impl<'a> NewtonRaphsonFunction<'a> {
  pub fn from_expr(
    expr: Expr,
    var: Var,
    function_table: &FunctionTable,
    simplifier: &'a dyn Simplifier,
  ) -> Result<Self, NewtonRaphsonError> {
    let deriv = differentiate(function_table, expr.clone(), var.clone())?;
    Ok(Self {
      function: ExprFunction::new(expr, var.clone(), simplifier),
      derivative: ExprFunction::new(deriv, var, simplifier),
    })
  }

  pub fn eval_at(&self, value: ComplexLike) -> Result<ComplexLike, NewtonRaphsonError> {
    let x = self.function.eval_at_complex(value)?;
    Ok(x)
  }

  pub fn eval_deriv_at(&self, value: ComplexLike) -> Result<ComplexLike, NewtonRaphsonError> {
    let x = self.derivative.eval_at_complex(value)?;
    Ok(x)
  }
}

impl Default for NewtonRaphsonMethod {
  fn default() -> Self {
    Self::new(
      Self::DEFAULT_EPSILON,
      Self::DEFAULT_MAX_ITERATIONS,
    )
  }
}

impl From<DifferentiationFailure> for NewtonRaphsonError {
  fn from(failure: DifferentiationFailure) -> Self {
    failure.error.into()
  }
}
