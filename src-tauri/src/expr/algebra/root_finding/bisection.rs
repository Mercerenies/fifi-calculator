
//! Implementation of the bisection method of root finding for
//! real-valued functions.
//!
//! See <https://en.wikipedia.org/wiki/Bisection_method>.

use super::FoundRoot;

use crate::expr::Expr;
use crate::expr::algebra::{ExprFunction, FunctionEvalError};
use crate::expr::simplifier::Simplifier;
use crate::expr::var::Var;
use crate::expr::number::Number;

use thiserror::Error;

#[derive(Debug)]
pub struct BisectionMethod {
  epsilon: f64,
  max_iterations: usize,
}

pub struct BisectionFunction<'a> {
  function: ExprFunction<'a>,
}

#[derive(Debug, Clone, Error)]
#[non_exhaustive]
pub enum BisectionError {
  #[error("{0}")]
  FunctionEvalError(#[from] FunctionEvalError),
  #[error("Failed to converge after {iterations} iterations")]
  FailedToConverge { iterations: usize },
  #[error("No root is contained between the two values {0} and {1}")]
  FailedToBisect(Number, Number),
  #[error("Bisection method requires two distinct inputs")]
  InputsAreIdentical,
}

impl BisectionMethod {
  pub const DEFAULT_EPSILON: f64 = 1e-5;
  pub const DEFAULT_MAX_ITERATIONS: usize = 1000;

  pub fn new(epsilon: f64, max_iterations: usize) -> Self {
    Self { epsilon, max_iterations }
  }

  pub fn find_root(
    &self,
    function: &BisectionFunction,
    mut left_bound: Number,
    mut right_bound: Number,
  ) -> Result<FoundRoot<Number>, BisectionError> {
    if left_bound == right_bound {
      return Err(BisectionError::InputsAreIdentical);
    }
    if left_bound > right_bound {
      return self.find_root(function, right_bound, left_bound);
    }

    // Verify that there is in fact a root contained between the two
    // values.
    let mut f_left = function.eval_at(left_bound.clone())?;
    let f_right = function.eval_at(right_bound.clone())?;
    if f_left.signum() == f_right.signum() {
      return Err(BisectionError::FailedToBisect(left_bound, right_bound));
    }

    for _ in 0..self.max_iterations {
      let pivot = (&left_bound + &right_bound) / Number::from(2);
      let f_pivot = function.eval_at(pivot.clone())?;
      let curr_epsilon = f_pivot.abs().to_f64().unwrap_or(f64::INFINITY);
      if curr_epsilon < self.epsilon {
        return Ok(FoundRoot { value: pivot, final_epsilon: curr_epsilon });
      }
      if f_pivot.signum() == f_left.signum() {
        left_bound = pivot;
        f_left = f_pivot;
      } else {
        right_bound = pivot;
        // f_right = f_pivot; // We never use f_right again :)
      }
    }
    Err(BisectionError::FailedToConverge { iterations: self.max_iterations })
  }
}

impl<'a> BisectionFunction<'a> {
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

impl Default for BisectionMethod {
  fn default() -> Self {
    Self::new(
      Self::DEFAULT_EPSILON,
      Self::DEFAULT_MAX_ITERATIONS,
    )
  }
}
