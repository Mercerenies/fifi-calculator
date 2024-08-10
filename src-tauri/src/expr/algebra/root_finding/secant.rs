
use super::FoundRoot;

use crate::expr::Expr;
use crate::expr::algebra::{ExprFunction, FunctionEvalError};
use crate::expr::simplifier::Simplifier;
use crate::expr::var::Var;
use crate::expr::number::Number;

use num::Zero;
use thiserror::Error;

#[derive(Debug)]
pub struct SecantMethod {
  epsilon: f64,
  max_iterations: usize,
}

pub struct SecantMethodFunction<'a> {
  function: ExprFunction<'a>,
}

#[derive(Debug, Clone, Error)]
#[non_exhaustive]
pub enum SecantMethodError {
  #[error("{0}")]
  FunctionEvalError(#[from] FunctionEvalError),
  #[error("Failed to converge after {iterations} iterations")]
  FailedToConverge { iterations: usize },
  #[error("Division by zero during Newton-Raphson convergence")]
  DivisionByZero,
}

impl SecantMethod {
  pub const DEFAULT_EPSILON: f64 = 1e-5;
  pub const DEFAULT_MAX_ITERATIONS: usize = 1000;

  pub fn new(epsilon: f64, max_iterations: usize) -> Self {
    Self { epsilon, max_iterations }
  }

  pub fn find_root(
    &self,
    function: &SecantMethodFunction,
    initial_value1: Number,
    initial_value2: Number,
  ) -> Result<FoundRoot<Number>, SecantMethodError> {
    let mut x0 = initial_value1;
    let mut x1 = initial_value2;
    for _ in 0..self.max_iterations {
      // First, check if we're already within epsilon of a root.
      let f1 = function.eval_at(x1.clone())?;
      let curr_epsilon = f1.abs().to_f64().unwrap_or(f64::INFINITY);
      if curr_epsilon < self.epsilon {
        return Ok(FoundRoot { value: x1, final_epsilon: curr_epsilon });
      }
      // Else, update current X values.
      let f0 = function.eval_at(x0.clone())?;
      let denominator = f1.clone() - f0;
      if denominator.is_zero() {
        return Err(SecantMethodError::DivisionByZero);
      }
      let numerator = f1 * (x1.clone() - x0);
      (x0, x1) = (x1.clone(), x1 - (numerator / denominator));
    }
    Err(SecantMethodError::FailedToConverge { iterations: self.max_iterations })
  }
}

impl<'a> SecantMethodFunction<'a> {
  pub fn new(function: ExprFunction<'a>) -> Self {
    Self { function }
  }

  pub fn from_expr(
    expr: Expr,
    var: Var,
    simplifier: &'a dyn Simplifier,
  ) -> Self {
    Self::new(ExprFunction::new(expr, var.clone(), simplifier))
  }

  pub fn eval_at(&self, value: Number) -> Result<Number, FunctionEvalError> {
    self.function.eval_at_real(value)
  }
}

impl Default for SecantMethod {
  fn default() -> Self {
    Self::new(
      Self::DEFAULT_EPSILON,
      Self::DEFAULT_MAX_ITERATIONS,
    )
  }
}
