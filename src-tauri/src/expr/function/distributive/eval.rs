
//! Configuration and implementation related to the distributive
//! property of certain operators.

use crate::expr::Expr;
use crate::expr::atom::Atom;
use crate::util::prism::ErrorWithPayload;

use thiserror::Error;

/// Errors that can occur during attempted application of the
/// distributive property.
#[derive(Debug, Clone, Error, PartialEq, Eq)]
#[error("{details}")]
pub struct DistributivePropertyError {
  pub details: DistributivePropertyErrorDetails,
  pub original_expr: Expr,
}

#[derive(Debug, Clone, Error, PartialEq, Eq)]
#[non_exhaustive]
pub enum DistributivePropertyErrorDetails {
  /// An argument index was provided, but the function does not have
  /// enough arguments.
  #[error("Argument index {0} is out of bounds")]
  ArgumentOutOfBounds(usize),
  /// The expression itself, or the argument over which distribution
  /// was required, was an atom rather than a compound expression.
  #[error("Expected a compound expression, but found {0:?}")]
  NotACompoundExpression(Atom),
}

impl DistributivePropertyError {
  pub fn new(details: DistributivePropertyErrorDetails, original_expr: Expr) -> Self {
    Self {
      details,
      original_expr,
    }
  }

  pub fn argument_out_of_bounds(index: usize, original_expr: Expr) -> Self {
    Self::new(DistributivePropertyErrorDetails::ArgumentOutOfBounds(index), original_expr)
  }

  pub fn not_a_compound_expr(atom: Atom, original_expr: Expr) -> Self {
    Self::new(DistributivePropertyErrorDetails::NotACompoundExpression(atom), original_expr)
  }
}

impl ErrorWithPayload<Expr> for DistributivePropertyError {
  fn recover_payload(self) -> Expr {
    self.original_expr
  }
}

/// Applies the distributive property to the given expression,
/// distributing the outermost expression over its `index`th argument.
///
/// Note that this function does not care what the heads of the
/// respective expressions are or if the distrbutive property makes
/// sense mathematically on those heads. This function merely
/// transposes the outermost expression into its `index`th argument
/// without regard for the arithmetic validity of the result.
pub fn distribute_over(expr: Expr, index: usize) -> Result<Expr, DistributivePropertyError> {
  let (f, args) = match expr {
    Expr::Call(f, args) => (f, args),
    Expr::Atom(atom) => {
      return Err(DistributivePropertyError::not_a_compound_expr(atom.clone(), Expr::from(atom)));
    },
  };
  let target_arg = match args.get(index) {
    Some(target_arg) => target_arg,
    None => {
      return Err(DistributivePropertyError::argument_out_of_bounds(index, Expr::call(f, args)));
    },
  };
  let (inner_f, inner_args) = match target_arg.clone() {
    Expr::Call(inner_f, inner_args) => (inner_f, inner_args),
    Expr::Atom(inner_atom) => {
      return Err(DistributivePropertyError::not_a_compound_expr(inner_atom, Expr::Call(f, args)));
    },
  };
  Ok(Expr::call(
    inner_f,
    inner_args
      .into_iter()
      .map(|inner_arg| Expr::call(f.clone(), substitute_nth_arg(args.clone(), index, inner_arg)))
      .collect(),
  ))
}

fn substitute_nth_arg<T>(mut args: Vec<T>, index: usize, new_arg: T) -> Vec<T> {
  args[index] = new_arg;
  args
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn test_substitute_nth() {
    let args = vec![10, 20, 30, 40, 50];
    assert_eq!(
      substitute_nth_arg(args.clone(), 2, 100),
      vec![10, 20, 100, 40, 50],
    );
  }

  #[test]
  #[should_panic]
  fn test_substitute_nth_out_of_bounds() {
    let args = vec![10, 20, 30, 40, 50];
    substitute_nth_arg(args, 5, 100);
  }

  #[test]
  fn test_distribute_over() {
    let expr = Expr::call("*", vec![
      Expr::from(10),
      Expr::call("+", vec![Expr::var("x").unwrap(), Expr::var("y").unwrap()]),
    ]);
    assert_eq!(
      distribute_over(expr, 1).unwrap(),
      Expr::call("+", vec![
        Expr::call("*", vec![Expr::from(10), Expr::var("x").unwrap()]),
        Expr::call("*", vec![Expr::from(10), Expr::var("y").unwrap()]),
      ]),
    );
  }

  #[test]
  fn test_distribute_over_on_atomic_expression() {
    let expr = Expr::from(9);
    assert_eq!(
      distribute_over(expr.clone(), 0).unwrap_err(),
      DistributivePropertyError::not_a_compound_expr(Atom::from(9), expr),
    );
  }

  #[test]
  fn test_distribute_over_on_atomic_argument() {
    let expr = Expr::call("*", vec![
      Expr::from(10),
      Expr::call("+", vec![Expr::var("x").unwrap(), Expr::var("y").unwrap()]),
    ]);
    assert_eq!(
      distribute_over(expr.clone(), 0).unwrap_err(),
      DistributivePropertyError::not_a_compound_expr(Atom::from(10), expr),
    );
  }

  #[test]
  fn test_distribute_over_argument_out_of_bounds() {
    let expr = Expr::call("*", vec![
      Expr::from(10),
      Expr::call("+", vec![Expr::var("x").unwrap(), Expr::var("y").unwrap()]),
    ]);
    assert_eq!(
      distribute_over(expr.clone(), 2).unwrap_err(),
      DistributivePropertyError::argument_out_of_bounds(2, expr),
    );
  }
}
