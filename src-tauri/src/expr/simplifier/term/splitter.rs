
use crate::expr::Expr;
use crate::expr::predicates;
use crate::expr::algebra::term::Term;
use crate::expr::algebra::factor::Factor;
use crate::expr::simplifier::base::{Simplifier, SimplifierContext};

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

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PartitionedTerm {
  pub literals: Term,
  pub others: Term,
}

impl TermPartialSplitter {
  pub fn new() -> Self {
    Self { _priv: () }
  }
}

impl Simplifier for TermPartialSplitter {
  fn simplify_expr_part(&self, expr: Expr, _ctx: &mut SimplifierContext) -> Expr {
    let term = Term::parse(expr);
    let PartitionedTerm { literals, others } = split_term(term);
    if literals.is_one() {
      others.into()
    } else if others.is_one() {
      literals.into()
    } else {
      Expr::call("*", vec![literals.into(), others.into()])
    }
  }
}

/// Partitions a [`Term`] into two subterms. The first consists of
/// "literal" values, more specifically those values which are likely
/// to be simplifiable via ordinary multiplication. The second
/// consists of everything else.
pub fn split_term(term: Term) -> PartitionedTerm {
  let (literals, others) = term.partition_factors(is_valid_multiplicand);
  let literals = literals.remove_ones();
  PartitionedTerm { literals, others }
}

/// Returns true if the expression represented by the argument
/// [`Factor`] can be simplified by multiplication and division. This
/// function corresponds exactly to the partial evaluation rules on
/// the multiplication operator.
///
/// Factors with a nontrivial exponent will always fail this
/// predicate.
fn is_valid_multiplicand(factor: &Factor) -> bool {
  if factor.exponent().is_some() {
    return false;
  }
  let expr = factor.base();
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
  fn test_sum_doesnt_simplify_with_splitter() {
    let expr = Expr::call("+", vec![Expr::from(1), Expr::from(2)]);
    let (new_expr, errors) = run_simplifier(&TermPartialSplitter::new(), expr.clone());
    assert!(errors.is_empty());
    assert_eq!(new_expr, expr);
  }

  #[test]
  fn test_partial_splitter_on_simple_product() {
    let expr = Expr::call("*", vec![Expr::from(3), Expr::from(4), var("x"), Expr::from(5)]);
    let (new_expr, errors) = run_simplifier(&TermPartialSplitter::new(), expr);
    assert!(errors.is_empty());
    assert_eq!(new_expr, Expr::call("*", vec![
      Expr::from(60),
      var("x"),
    ]));
  }

  #[test]
  fn test_partial_splitter_on_product_of_several_terms() {
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
      Expr::from(6),
      Expr::call("*", vec![var("x"), var("y")]),
    ]));
  }

  #[test]
  fn test_split_on_product_of_several_terms_with_a_one_in_denominator() {
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
    assert_eq!(new_expr, Expr::call("*", vec![
      Expr::from(6),
      Expr::call("*", vec![var("x"), var("y")]),
    ]));
  }

  #[test]
  fn test_fraction_partial_split_evaluation() {
    let expr = Expr::call("/", vec![
      Expr::call("*", vec![
        Expr::from(4),
        Expr::from(5),
        var("x"),
        Expr::from(6),
        var("y"),
      ]),
      Expr::call("*", vec![
        Expr::from(1),
        Expr::from(2),
        var("z"),
        Expr::from(3),
        var("t"),
      ]),
    ]);
    let (new_expr, errors) = run_simplifier(&TermPartialSplitter::new(), expr);
    assert!(errors.is_empty());
    assert_eq!(new_expr, Expr::call("*", vec![
      Expr::from(20),
      Expr::call("/", vec![
        Expr::call("*", vec![var("x"), var("y")]),
        Expr::call("*", vec![var("z"), var("t")]),
      ]),
    ]));
  }

  #[test]
  fn test_fraction_partial_split_with_all_terms_scalar() {
    let expr = Expr::call("/", vec![
      Expr::call("*", vec![
        Expr::from(5),
        Expr::from(6),
        Expr::from(7),
        Expr::from(8),
      ]),
      Expr::call("*", vec![
        Expr::from(2),
        Expr::from(3),
        Expr::from(4),
      ]),
    ]);
    let (new_expr, errors) = run_simplifier(&TermPartialSplitter::new(), expr.clone());
    assert!(errors.is_empty());
    assert_eq!(new_expr, Expr::from(70));
  }

  #[test]
  fn test_fraction_partial_split_with_all_terms_non_scalar() {
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
