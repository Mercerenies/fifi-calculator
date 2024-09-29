
//! Defines simplifiers which operate at the level of a single
//! polynomial consisting of zero or more terms.
//!
//! See [`TermSorter`] for more details.

use crate::expr::Expr;
use crate::expr::ordering::cmp_expr;
use crate::expr::arithmetic::ArithExpr;
use crate::expr::algebra::polynomial::{Polynomial, parse_polynomial};
use crate::expr::algebra::term::{Sign, SignedTerm, Term};
use crate::expr::algebra::factor::Factor;
use crate::expr::simplifier::base::{Simplifier, SimplifierContext};
use crate::expr::simplifier::term::{PartitionedTerm, split_term};
use crate::util::cmp_iter_by;

use itertools::Itertools;

use std::cmp::Ordering;

#[derive(Debug, Default)]
pub struct TermSorter {
  _priv: (),
}

impl TermSorter {
  pub fn new() -> Self {
    Self { _priv: () }
  }
}

impl Simplifier for TermSorter {
  fn simplify_expr_part(&self, expr: Expr, _ctx: &mut SimplifierContext) -> Expr {
    let polynomial = parse_polynomial(expr);
    let sorted_terms = group_and_sort_terms(polynomial);
    let sorted_polynomial = Polynomial::from_terms(sorted_terms);
    sorted_polynomial.into()
  }
}

fn group_and_sort_terms(terms: impl IntoIterator<Item = SignedTerm>) -> Vec<SignedTerm> {
  fn group(signed_term: SignedTerm) -> (Term, SignedTerm) {
    let PartitionedTerm { literals, others } = split_term(signed_term.term);
    (others, SignedTerm::new(signed_term.sign, literals))
  }

  let grouped_terms = terms.into_iter().map(group).into_group_map();
  let mut terms_seq: Vec<(Term, ArithExpr)> = grouped_terms.into_iter().map(|(key, coeffs)| {
    (key, ArithExpr::sum(coeffs))
  }).collect();
  terms_seq.sort_by(|a, b| cmp_term(&a.0, &b.0));
  terms_seq.into_iter().map(|(key, arith_expr)| {
    let product = Term::parse(arith_expr.into()) * key;
    SignedTerm::new(Sign::Positive, product.remove_ones())
  }).collect()
}

/// Compares two terms by comparing their numerators and denominators
/// lexicographically using [`cmp_expr`], except that factors which
/// lack an explicit exponent are treated as having an exponent of one
/// for the purposes of this comparison.
fn cmp_term(a: &Term, b: &Term) -> Ordering {
  cmp_iter_by(a.numerator(), b.numerator(), |a, b| cmp_factor(a, b))
    .then_with(|| cmp_iter_by(a.denominator(), b.denominator(), |a, b| cmp_factor(a, b)))
}

fn cmp_factor(a: &Factor, b: &Factor) -> Ordering {
  cmp_expr(a.base(), b.base())
    .then_with(|| cmp_expr(&a.exponent_or_one(), &b.exponent_or_one()))
}

#[cfg(test)]
mod tests {
  use super::*;
  use crate::expr::simplifier::test_utils::{run_simplifier_no_errors};

  fn var(name: &str) -> Expr {
    Expr::var(name).unwrap()
  }

  #[test]
  fn test_term_sorter_on_simple_exprs() {
    let sorter = TermSorter::new();

    let expr = Expr::from(10);
    assert_eq!(run_simplifier_no_errors(&sorter, expr.clone()), expr);

    let expr = var("x");
    assert_eq!(run_simplifier_no_errors(&sorter, expr.clone()), expr);

    let expr = Expr::call("example_function", vec![Expr::from(1), Expr::from(2)]);
    assert_eq!(run_simplifier_no_errors(&sorter, expr.clone()), expr);
  }

  #[test]
  fn test_term_sorter_on_distinct_terms() {
    let sorter = TermSorter::new();

    let expr = Expr::call("+", vec![var("x"), var("y")]);
    assert_eq!(run_simplifier_no_errors(&sorter, expr.clone()), expr);

    let expr = Expr::call("+", vec![var("y"), var("x")]);
    assert_eq!(
      run_simplifier_no_errors(&sorter, expr),
      Expr::call("+", vec![var("x"), var("y")]),
    );
  }

  #[test]
  fn test_term_sorter_on_distinct_terms_with_negative_results() {
    let sorter = TermSorter::new();

    let expr = Expr::call("-", vec![var("y"), var("x")]);
    assert_eq!(
      run_simplifier_no_errors(&sorter, expr),
      Expr::call("+", vec![
        Expr::call("*", vec![Expr::from(-1), var("x")]),
        var("y"),
      ]),
    );

    let expr = Expr::call("-", vec![var("x"), var("y")]);
    // TODO Can we get this down to just be x - y rather than x + (-1) * y?
    assert_eq!(
      run_simplifier_no_errors(&sorter, expr),
      Expr::call("+", vec![
        var("x"),
        Expr::call("*", vec![Expr::from(-1), var("y")]),
      ]),
    );

    let expr = Expr::call("+", vec![
      Expr::call("negate", vec![var("x")]),
      var("y"),
      var("z"),
    ]);
    assert_eq!(
      run_simplifier_no_errors(&sorter, expr),
      Expr::call("+", vec![
        Expr::call("*", vec![Expr::from(-1), var("x")]),
        var("y"),
        var("z"),
      ]),
    );

    let expr = Expr::call("+", vec![
      var("x"),
      Expr::call("negate", vec![var("y")]),
      var("z"),
    ]);
    assert_eq!(
      run_simplifier_no_errors(&sorter, expr),
      Expr::call("+", vec![
        var("x"),
        Expr::call("*", vec![Expr::from(-1), var("y")]),
        var("z"),
      ]),
    );

    let expr = Expr::call("+", vec![
      var("x"),
      var("y"),
      Expr::call("negate", vec![var("z")]),
    ]);
    assert_eq!(
      run_simplifier_no_errors(&sorter, expr),
      Expr::call("+", vec![
        var("x"),
        var("y"),
        Expr::call("*", vec![Expr::from(-1), var("z")]),
      ]),
    );
  }

  #[test]
  fn test_term_sorter_on_several_terms() {
    let sorter = TermSorter::new();

    let expr = Expr::call("+", vec![
      var("y"),
      var("x"),
      Expr::call("*", vec![var("z"), Expr::from(3)]),
    ]);
    assert_eq!(
      run_simplifier_no_errors(&sorter, expr),
      Expr::call("+", vec![
        var("x"),
        var("y"),
        Expr::call("*", vec![Expr::from(3), var("z")]),
      ]),
    );

    let expr = Expr::call("+", vec![
      var("y"),
      var("x"),
      Expr::call("*", vec![
        var("z"),
        Expr::from(3),
        Expr::from(2),
      ]),
    ]);
    assert_eq!(
      run_simplifier_no_errors(&sorter, expr),
      Expr::call("+", vec![
        var("x"),
        var("y"),
        Expr::call("*", vec![Expr::from(6), var("z")]),
      ]),
    );
  }

  #[test]
  fn test_term_sorter_on_two_x_terms() {
    let sorter = TermSorter::new();

    let expr = Expr::call("+", vec![
      Expr::call("*", vec![var("x"), Expr::from(10)]),
      Expr::call("*", vec![var("x"), Expr::from(5)]),
    ]);
    assert_eq!(
      run_simplifier_no_errors(&sorter, expr),
      Expr::call("*", vec![Expr::from(15), var("x")]),
    );

    let expr = Expr::call("-", vec![
      Expr::call("*", vec![var("x"), Expr::from(10)]),
      var("x"),
    ]);
    assert_eq!(
      run_simplifier_no_errors(&sorter, expr),
      Expr::call("*", vec![Expr::from(9), var("x")]),
    );
  }

  #[test]
  fn test_term_sorter_on_x_terms_of_varying_powers() {
    let sorter = TermSorter::new();

    let expr = Expr::call("+", vec![
      Expr::call("*", vec![var("x"), Expr::from(10)]),
      Expr::call("*", vec![Expr::call("^", vec![var("x"), Expr::from(2)]), Expr::from(5)]),
      Expr::call("*", vec![var("x"), Expr::from(5)]),
      Expr::call("*", vec![Expr::call("^", vec![var("x"), Expr::from(2)]), Expr::from(2)]),
      Expr::call("^", vec![var("x"), Expr::from(3)]),
      Expr::call("^", vec![Expr::from(99), var("x")]),
    ]);
    assert_eq!(
      run_simplifier_no_errors(&sorter, expr),
      Expr::call("+", vec![
        Expr::call("^", vec![Expr::from(99), var("x")]),
        Expr::call("*", vec![Expr::from(15), var("x")]),
        Expr::call("*", vec![Expr::from(7), Expr::call("^", vec![var("x"), Expr::from(2)])]),
        Expr::call("^", vec![var("x"), Expr::from(3)]),
      ]),
    );
  }

  #[test]
  fn test_term_sorter_on_complex_x_terms() {
    let sorter = TermSorter::new();

    let expr = Expr::call("+", vec![
      Expr::call("*", vec![var("x"), Expr::from(10)]),
      Expr::call("*", vec![var("x"), Expr::from(5)]),
      Expr::call("-", vec![
        var("x"),
        Expr::call("*", vec![Expr::from(3), var("x")]),
      ]),
      Expr::call("negate", vec![
        Expr::call("*", vec![Expr::from(7), var("x")]),
      ]),
    ]);
    assert_eq!(
      run_simplifier_no_errors(&sorter, expr),
      Expr::call("*", vec![Expr::from(6), var("x")]),
    );

    let expr = Expr::call("+", vec![
      Expr::call("*", vec![var("x"), Expr::from(-10)]),
      Expr::call("*", vec![var("x"), Expr::from(5)]),
      Expr::call("-", vec![
        var("x"),
        Expr::call("*", vec![Expr::from(3), var("x")]),
      ]),
      Expr::call("negate", vec![
        Expr::call("*", vec![Expr::from(7), var("x")]),
      ]),
    ]);
    assert_eq!(
      run_simplifier_no_errors(&sorter, expr),
      Expr::call("*", vec![Expr::from(-14), var("x")]),
    );
  }

  #[test]
  fn test_term_sorter_on_complex_mixed_terms() {
    let sorter = TermSorter::new();

    let expr = Expr::call("+", vec![
      Expr::call("*", vec![var("x"), Expr::from(10)]),
      Expr::call("*", vec![var("y"), Expr::from(5)]),
      Expr::call("-", vec![
        var("x"),
        Expr::call("*", vec![Expr::from(3), var("y")]),
      ]),
      Expr::call("negate", vec![
        Expr::call("*", vec![Expr::from(7), var("x")]),
      ]),
    ]);
    assert_eq!(
      run_simplifier_no_errors(&sorter, expr),
      Expr::call("+", vec![
        Expr::call("*", vec![Expr::from(4), var("x")]),
        Expr::call("*", vec![Expr::from(2), var("y")]),
      ]),
    );

    let expr = Expr::call("+", vec![
      Expr::call("*", vec![var("x"), Expr::from(10)]),
      Expr::call("*", vec![var("y"), Expr::from(11)]),
      Expr::call("-", vec![
        var("x"),
        Expr::call("*", vec![Expr::from(3), var("y")]),
      ]),
      Expr::call("negate", vec![
        Expr::call("*", vec![Expr::from(7), var("x")]),
      ]),
    ]);
    assert_eq!(
      run_simplifier_no_errors(&sorter, expr),
      Expr::call("+", vec![
        Expr::call("*", vec![Expr::from(4), var("x")]),
        Expr::call("*", vec![Expr::from(8), var("y")]),
      ]),
    );
  }

  #[test]
  fn test_term_sorter_on_complex_mixed_terms_with_result_coefficient_zero() {
    let sorter = TermSorter::new();

    let expr = Expr::call("+", vec![
      Expr::call("*", vec![var("x"), Expr::from(10)]),
      Expr::call("*", vec![var("y"), Expr::from(3)]),
      Expr::call("-", vec![
        var("x"),
        Expr::call("*", vec![Expr::from(3), var("y")]),
      ]),
      Expr::call("negate", vec![
        Expr::call("*", vec![Expr::from(7), var("x")]),
      ]),
    ]);
    assert_eq!(
      run_simplifier_no_errors(&sorter, expr),
      Expr::call("+", vec![
        Expr::call("*", vec![Expr::from(4), var("x")]),
        Expr::call("*", vec![Expr::from(0), var("y")]),
      ]),
    );
  }

  #[test]
  fn test_term_sorter_on_complex_mixed_terms_with_result_coefficient_one() {
    let sorter = TermSorter::new();

    let expr = Expr::call("+", vec![
      Expr::call("*", vec![var("x"), Expr::from(10)]),
      Expr::call("*", vec![var("y"), Expr::from(4)]),
      Expr::call("-", vec![
        var("x"),
        Expr::call("*", vec![Expr::from(3), var("y")]),
      ]),
      Expr::call("negate", vec![
        Expr::call("*", vec![Expr::from(7), var("x")]),
      ]),
    ]);
    assert_eq!(
      run_simplifier_no_errors(&sorter, expr),
      Expr::call("+", vec![
        Expr::call("*", vec![Expr::from(4), var("x")]),
        var("y"),
      ]),
    );

    let expr = Expr::call("+", vec![
      Expr::call("*", vec![var("x"), Expr::from(10)]),
      Expr::call("*", vec![var("y"), Expr::from(5)]),
      Expr::call("-", vec![
        var("x"),
        Expr::call("*", vec![Expr::from(3), var("y")]),
      ]),
      Expr::call("negate", vec![
        Expr::call("*", vec![Expr::from(10), var("x")]),
      ]),
    ]);
    assert_eq!(
      run_simplifier_no_errors(&sorter, expr),
      Expr::call("+", vec![
        var("x"),
        Expr::call("*", vec![Expr::from(2), var("y")]),
      ]),
    );
  }
}
