
//! Defines a simplifier which uses the [`Term`] abstraction to
//! perform simplification.
//!
//! See [`TermPartialSplitter`] for more details.

use crate::expr::algebra::term::TermParser;
use crate::expr::predicates;
use crate::expr::Expr;
use super::base::{Simplifier, SimplifierContext};

use num::One;

/// `TermPartialSplitter` is a [`Simplifier`] that translates the
/// target expression into a [`Term`] and then tries to partially
/// evaluate any numerical constants in the term.
///
/// Translation into a `Term` has the immediate side benefit of
/// simplifying nested fractions such as `1 / (1 / x)`.
#[derive(Debug, Default)]
pub struct TermPartialSplitter {
  _priv: (),
}

impl TermPartialSplitter {
  pub fn new() -> Self {
    Self { _priv: () }
  }
}

impl Simplifier for TermPartialSplitter {
  fn simplify_expr_part(&self, expr: Expr, _ctx: &mut SimplifierContext) -> Expr {
    let term = TermParser::new().parse(expr);
    let (literals, others) = term.partition_factors(is_valid_multiplicand);
    let literals = literals.remove_ones();
    if literals.is_one() {
      others.into()
    } else if others.is_one() {
      literals.into()
    } else {
      Expr::call("*", vec![literals.into(), others.into()])
    }
  }
}

/// Returns true if the expression can be simplified by multiplication
/// and division. This function corresponds exactly to the partial
/// evaluation rules on the multiplication operator.
fn is_valid_multiplicand(expr: &Expr) -> bool {
  predicates::is_tensor(expr) ||
    predicates::is_complex_or_inf(expr) ||
    predicates::is_unbounded_interval_like(expr)
}

#[cfg(test)]
mod tests {
  use super::*;
  use crate::expr::simplifier::test_utils::run_simplifier;
  use crate::expr::Expr;

  fn var(name: &str) -> Expr {
    Expr::var(name).unwrap()
  }

  #[test]
  fn test_sum_doesnt_simplify() {
    let expr = Expr::call("+", vec![Expr::from(1), Expr::from(2)]);
    let (new_expr, errors) = run_simplifier(&TermPartialSplitter::new(), expr.clone());
    assert!(errors.is_empty());
    assert_eq!(new_expr, expr);
  }

  #[test]
  fn test_simple_product() {
    let expr = Expr::call("*", vec![Expr::from(3), Expr::from(4), var("x"), Expr::from(5)]);
    let (new_expr, errors) = run_simplifier(&TermPartialSplitter::new(), expr);
    assert!(errors.is_empty());
    assert_eq!(new_expr, Expr::call("*", vec![
      Expr::call("*", vec![Expr::from(3), Expr::from(4), Expr::from(5)]),
      var("x"),
    ]));
  }

  #[test]
  fn test_product_of_several_terms() {
    let expr = Expr::call("*", vec![
      Expr::from(1),
      Expr::from(2),
      var("x"),
      Expr::from(3),
      var("y"),
    ]);
    let (new_expr, errors) = run_simplifier(&TermPartialSplitter::new(), expr);
    assert!(errors.is_empty());
    assert_eq!(new_expr, Expr::call("*", vec![
      Expr::call("*", vec![Expr::from(2), Expr::from(3)]), // Note: The 1 is eliminated from the numerator.
      Expr::call("*", vec![var("x"), var("y")]),
    ]));
  }

  #[test]
  fn test_product_of_several_terms_with_a_one_in_denominator() {
    let expr = Expr::call("/", vec![
      Expr::call("*", vec![
        Expr::from(1),
        Expr::from(2),
        var("x"),
        Expr::from(3),
        var("y"),
      ]),
      Expr::call("*", vec![Expr::from(1), Expr::from(1)]),
    ]);
    let (new_expr, errors) = run_simplifier(&TermPartialSplitter::new(), expr);
    assert!(errors.is_empty());
    // Note: All '1's are eliminated.
    assert_eq!(new_expr, Expr::call("*", vec![
      Expr::call("*", vec![Expr::from(2), Expr::from(3)]),
      Expr::call("*", vec![var("x"), var("y")]),
    ]));
  }

  #[test]
  fn test_fraction_partial_evaluation() {
    let expr = Expr::call("/", vec![
      Expr::call("*", vec![
        Expr::from(1),
        Expr::from(2),
        var("x"),
        Expr::from(3),
        var("y"),
      ]),
      Expr::call("*", vec![
        Expr::from(4),
        Expr::from(5),
        var("z"),
        Expr::from(6),
        var("t"),
      ]),
    ]);
    let (new_expr, errors) = run_simplifier(&TermPartialSplitter::new(), expr);
    assert!(errors.is_empty());
    assert_eq!(new_expr, Expr::call("*", vec![
      Expr::call("/", vec![
        Expr::call("*", vec![Expr::from(2), Expr::from(3)]), // Note: The 1 is eliminated from the numerator.
        Expr::call("*", vec![Expr::from(4), Expr::from(5), Expr::from(6)]),
      ]),
      Expr::call("/", vec![
        Expr::call("*", vec![var("x"), var("y")]),
        Expr::call("*", vec![var("z"), var("t")]),
      ]),
    ]));
  }

  #[test]
  fn test_fraction_with_all_terms_scalar() {
    let expr = Expr::call("/", vec![
      Expr::call("*", vec![
        Expr::from(2),
        Expr::from(3),
        Expr::from(4),
      ]),
      Expr::call("*", vec![
        Expr::from(5),
        Expr::from(6),
        Expr::from(7),
      ]),
    ]);
    let (new_expr, errors) = run_simplifier(&TermPartialSplitter::new(), expr.clone());
    assert!(errors.is_empty());
    assert_eq!(new_expr, expr);
  }

  #[test]
  fn test_fraction_with_all_terms_non_scalar() {
    let expr = Expr::call("/", vec![
      Expr::call("*", vec![
        var("x"),
        Expr::call("+", vec![Expr::from(1), Expr::from(2)]),
      ]),
      var("y"),
    ]);
    let (new_expr, errors) = run_simplifier(&TermPartialSplitter::new(), expr.clone());
    assert!(errors.is_empty());
    assert_eq!(new_expr, expr);
  }
}
