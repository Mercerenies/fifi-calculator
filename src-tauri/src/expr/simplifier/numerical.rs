
use super::base::{Simplifier, SimplifierContext};
use super::chained::ChainedSimplifier;
use crate::expr::Expr;
use crate::expr::atom::Atom;

/// A [`Simplifier`] which replaces any values with their "inexact'
/// equivalents.
///
/// Currently, this only replaces non-integer rational numbers with
/// their floating-point equivalents (via
/// [`Number::ratio_to_inexact`]). In the future, it may be extended
/// as we support more symbolic treatments of names like `e` and `pi`.
///
/// This simplifier is NOT run as part of the standard simplifier
/// pipeline.
#[derive(Debug, Default)]
pub struct NumericalSimplifier {
  _priv: (),
}

impl NumericalSimplifier {
  pub fn new() -> Self {
    Self::default()
  }

  /// Constructs a [`ChainedSimplifier`] which runs a
  /// [`NumericalSimplifier`] step after the given simplifier.
  pub fn appended<'a, S>(base_simplifier: S) -> ChainedSimplifier<'a, 'static>
  where S: Simplifier + 'a {
    ChainedSimplifier::new(
      Box::new(base_simplifier),
      Box::new(Self::new()),
    )
  }
}

impl Simplifier for NumericalSimplifier {
  fn simplify_expr_part(&self, expr: Expr, _ctx: &mut SimplifierContext) -> Expr {
    simplify_numerically(expr)
  }
}

pub fn simplify_numerically(expr: Expr) -> Expr {
  match expr {
    Expr::Atom(Atom::Number(n)) => Expr::from(n.ratio_to_inexact()),
    expr => expr,
  }
}

#[cfg(test)]
mod tests {
  use super::*;
  use crate::expr::simplifier::test_utils::run_simplifier_no_errors;
  use crate::expr::number::Number;
  use crate::assert_strict_eq;

  #[test]
  fn test_on_simple_atoms() {
    let simplifier = NumericalSimplifier::new();

    let expr = Expr::from("ABC");
    assert_strict_eq!(run_simplifier_no_errors(&simplifier, expr), Expr::from("ABC"));
    let expr = Expr::var("x").unwrap();
    assert_strict_eq!(run_simplifier_no_errors(&simplifier, expr), Expr::var("x").unwrap());
    let expr = Expr::from(3);
    assert_strict_eq!(run_simplifier_no_errors(&simplifier, expr), Expr::from(3));
    let expr = Expr::from(Number::ratio(1, 2));
    assert_strict_eq!(run_simplifier_no_errors(&simplifier, expr), Expr::from(0.5));
    let expr = Expr::from(0.5);
    assert_strict_eq!(run_simplifier_no_errors(&simplifier, expr), Expr::from(0.5));
  }

  #[test]
  fn test_on_call_expr() {
    let simplifier = NumericalSimplifier::new();

    let expr = Expr::call("foobar", vec![
      Expr::from("ABC"),
      Expr::from(10),
      Expr::from(Number::ratio(1, 2)),
      Expr::from(0.2),
      Expr::from(Number::ratio(1, -2)),
    ]);
    assert_strict_eq!(
      run_simplifier_no_errors(&simplifier, expr),
      Expr::call("foobar", vec![
        Expr::from("ABC"),
        Expr::from(10),
        Expr::from(0.5),
        Expr::from(0.2),
        Expr::from(-0.5),
      ]),
    );
  }
}
