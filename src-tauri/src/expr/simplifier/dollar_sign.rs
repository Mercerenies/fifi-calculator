
use super::base::{Simplifier, SimplifierContext};
use super::error::SimplifierError;
use super::chained::ChainedSimplifier;
use crate::stack::StackError;
use crate::stack::base::RandomAccessStackLike;
use crate::expr::Expr;
use crate::expr::atom::Atom;
use crate::expr::var::dollar_sign::DollarSignVar;

use thiserror::Error;

use std::convert::TryFrom;

/// A [`Simplifier`] which replaces any dollar-sign variables (as per
/// [`DollarSignVar`]) with an appropriate value from the stack.
/// Dollar-sign variables are indexed from the top of the stack, with
/// 1 being the top index.
///
/// Attempting to index into the stack position 0, or a stack position
/// greater than the length of the stack, produces a
/// [`SimplifierError`].
#[derive(Debug)]
pub struct DollarSignRefSimplifier<'a, S> {
  stack: &'a S,
}

#[derive(Debug, Clone, Error)]
#[non_exhaustive]
pub enum DollarSignRefError {
  #[error("Attempt to substitute index 0 from the stack")]
  IndexZero,
  #[error("{0}")]
  StackError(#[from] StackError),
}

impl<'a, S> DollarSignRefSimplifier<'a, S>
where S: RandomAccessStackLike<Elem = Expr> {
  /// Creates a new [`DollarSignRefSimplifier`] referencing the given
  /// stack.
  pub fn new(stack: &'a S) -> Self {
    Self { stack }
  }

  /// Constructs a [`ChainedSimplifier`] which runs a
  /// [`DollarSignRefSimplifier`] step before the given simplifier.
  pub fn prepended<'b, SIMPL>(stack: &'a S, base_simplifier: SIMPL) -> ChainedSimplifier<'a, 'b>
  where SIMPL: Simplifier + 'b {
    ChainedSimplifier::new(
      Box::new(Self::new(stack)),
      Box::new(base_simplifier),
    )
  }

  fn substitute_var(&self, var: &DollarSignVar) -> Result<Expr, DollarSignRefError> {
    if var.value() == 0 {
      return Err(DollarSignRefError::IndexZero);
    }
    let index = (var.value() - 1) as i64;
    let expr = self.stack.get(index)?;
    Ok(expr.to_owned())
  }
}

impl<S> Simplifier for DollarSignRefSimplifier<'_, S>
where S: RandomAccessStackLike<Elem = Expr> {
  fn simplify_expr_part(&self, expr: Expr, ctx: &mut SimplifierContext) -> Expr {
    let Some(var) = try_into_dollar_sign_var(&expr) else {
      return expr;
    };
    match self.substitute_var(&var) {
      Ok(new_expr) => new_expr,
      Err(err) => {
        ctx.errors.push(SimplifierError::new(var.to_string(), err));
        expr
      }
    }
  }
}

fn try_into_dollar_sign_var(expr: &Expr) -> Option<DollarSignVar> {
  let Expr::Atom(Atom::Var(var)) = expr else {
    return None;
  };
  DollarSignVar::try_from(var).ok()
}

#[cfg(test)]
mod tests {
  use super::*;
  use crate::stack::Stack;
  use crate::expr::simplifier::test_utils::run_simplifier;

  fn example_stack() -> Stack<Expr> {
    Stack::from(vec![Expr::from(100), Expr::from(200), Expr::from(300)])
  }

  fn var(s: &str) -> Expr {
    Expr::var(s).unwrap()
  }

  #[test]
  fn test_dollar_sign_ref_simplifier() {
    let stack = example_stack();
    let expr = Expr::call("foobar", vec![var("$2"), var("$1"), var("$1"), var("xx")]);
    let simplifier = DollarSignRefSimplifier::new(&stack);
    let (new_expr, errors) = run_simplifier(&simplifier, expr);

    assert!(errors.is_empty());
    assert_eq!(new_expr, Expr::call("foobar", vec![
      Expr::from(200),
      Expr::from(300),
      Expr::from(300),
      var("xx"),
    ]));
  }

  #[test]
  fn test_dollar_sign_ref_simplifier_index_zero() {
    let stack = example_stack();
    let expr = Expr::call("foobar", vec![var("$0"), var("$1"), var("$1"), var("xx")]);
    let simplifier = DollarSignRefSimplifier::new(&stack);
    let (new_expr, errors) = run_simplifier(&simplifier, expr);
    let errors = errors.into_vec();

    assert_eq!(errors.len(), 1);
    assert_eq!(errors[0].to_string(), "$0: Attempt to substitute index 0 from the stack");
    assert_eq!(new_expr, Expr::call("foobar", vec![
      var("$0"),
      Expr::from(300),
      Expr::from(300),
      var("xx"),
    ]));
  }

  #[test]
  fn test_dollar_sign_ref_simplifier_index_out_of_bounds() {
    let stack = example_stack();
    let expr = Expr::call("foobar", vec![var("$1"), var("$2"), var("$3"), var("$4")]);
    let simplifier = DollarSignRefSimplifier::new(&stack);
    let (new_expr, errors) = run_simplifier(&simplifier, expr);
    let errors = errors.into_vec();

    assert_eq!(errors.len(), 1);
    assert_eq!(errors[0].to_string(), "$4: Not enough stack elements, expected at least 4 but found 3.");
    assert_eq!(new_expr, Expr::call("foobar", vec![
      Expr::from(300),
      Expr::from(200),
      Expr::from(100),
      var("$4"),
    ]));
  }
}
