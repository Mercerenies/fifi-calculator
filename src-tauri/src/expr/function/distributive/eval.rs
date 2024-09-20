
//! Configuration and implementation related to the distributive
//! property of certain operators.

use crate::expr::Expr;
use crate::expr::atom::Atom;

use thiserror::Error;

/// Errors that can occur during attempted application of the
/// distributive property.
#[derive(Debug, Clone, Error, PartialEq, Eq)]
#[non_exhaustive]
pub enum DistributivePropertyError {
  /// An argument index was provided, but the function does not have
  /// enough arguments.
  #[error("Argument index {0} is out of bounds")]
  ArgumentOutOfBounds(usize),
  /// The expression itself, or the argument over which distribution
  /// was required, was an atom rather than a compound expression.
  #[error("Expected a compound expression, but found {0:?}")]
  NotACompoundExpression(Atom),
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
  let (f, args) = as_compound_expr(expr)?;
  let target_arg = args.get(index).ok_or_else(|| DistributivePropertyError::ArgumentOutOfBounds(index))?;
  let (inner_f, inner_args) = as_compound_expr(target_arg.clone())?;
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

fn as_compound_expr(expr: Expr) -> Result<(String, Vec<Expr>), DistributivePropertyError> {
  match expr {
    Expr::Atom(atom) => Err(DistributivePropertyError::NotACompoundExpression(atom)),
    Expr::Call(f, args) => Ok((f, args)),
  }
}

#[cfg(test)]
mod tests {
  use super::*;
  use crate::expr::var::Var;

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
  fn test_as_compound_expr() {
    let (f, args) = as_compound_expr(Expr::call("f", vec![Expr::from(0), Expr::from(1)])).unwrap();
    assert_eq!(f, "f");
    assert_eq!(args, vec![Expr::from(0), Expr::from(1)]);
  }

  #[test]
  fn test_as_compound_expr_on_atoms() {
    assert_eq!(
      as_compound_expr(Expr::from(0)).unwrap_err(),
      DistributivePropertyError::NotACompoundExpression(Atom::from(0)),
    );
    assert_eq!(
      as_compound_expr(Expr::string("abc")).unwrap_err(),
      DistributivePropertyError::NotACompoundExpression(Atom::String(String::from("abc"))),
    );
    assert_eq!(
      as_compound_expr(Expr::var("x").unwrap()).unwrap_err(),
      DistributivePropertyError::NotACompoundExpression(Atom::Var(Var::new("x").unwrap())),
    );
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
      distribute_over(expr, 0).unwrap_err(),
      DistributivePropertyError::NotACompoundExpression(Atom::from(9)),
    );
  }

  #[test]
  fn test_distribute_over_on_atomic_argument() {
    let expr = Expr::call("*", vec![
      Expr::from(10),
      Expr::call("+", vec![Expr::var("x").unwrap(), Expr::var("y").unwrap()]),
    ]);
    assert_eq!(
      distribute_over(expr, 0).unwrap_err(),
      DistributivePropertyError::NotACompoundExpression(Atom::from(10)),
    );
  }

  #[test]
  fn test_distribute_over_argument_out_of_bounds() {
    let expr = Expr::call("*", vec![
      Expr::from(10),
      Expr::call("+", vec![Expr::var("x").unwrap(), Expr::var("y").unwrap()]),
    ]);
    assert_eq!(
      distribute_over(expr, 2).unwrap_err(),
      DistributivePropertyError::ArgumentOutOfBounds(2),
    );
  }
}
