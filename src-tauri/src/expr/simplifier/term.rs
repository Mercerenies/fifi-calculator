
//! Defines simplifiers which use the [`Term`] abstraction to perform
//! simplification.
//!
//! See [`TermPartialSplitter`] and [`FactorSorter`] for more details.

use crate::expr::algebra::term::TermParser;
use crate::expr::algebra::factor::Factor;
use crate::expr::predicates;
use crate::expr::Expr;
use crate::expr::ordering::cmp_expr;
use super::base::{Simplifier, SimplifierContext};

use num::One;
use itertools::Itertools;

use std::cmp::Ordering;

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

/// `FactorSorter` is a [`Simplifier`] that orders the factors in the
/// numerator and denominator of a term according to a sensible and
/// canonical ordering. This simplifier also groups factors with the
/// same exponential base.
#[derive(Debug, Default)]
pub struct FactorSorter {
  _priv: (),
}

impl TermPartialSplitter {
  pub fn new() -> Self {
    Self { _priv: () }
  }
}

impl FactorSorter {
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

impl Simplifier for FactorSorter {
  fn simplify_expr_part(&self, expr: Expr, _ctx: &mut SimplifierContext) -> Expr {
    let term_parser = TermParser::new();
    let term = term_parser.parse(expr);
    let (numer, denom) = term.into_parts_as_factors();
    let mut numer = group_and_sort_factors(numer);
    let mut denom = group_and_sort_factors(denom);
    move_common_terms_to_numer(&mut numer, &mut denom);
    term_parser.from_parts(numer, denom).into()
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

fn group_and_sort_factors(factors: Vec<Factor>) -> Vec<Factor> {
  let grouped_factors = factors.into_iter()
    .map(Factor::into_parts)
    .into_group_map();
  let mut factors: Vec<_> = grouped_factors.into_iter()
    .map(|(base, exponents)| {
      let exponents = exponents.into_iter()
        .map(|e| e.unwrap_or_else(Expr::one))
        .collect();
      Factor::from_parts(base, sum(exponents))
        .simplify_trivial_powers()
    })
    .collect();
  factors.sort_by(cmp_factor);
  factors
}

fn sum(mut exprs: Vec<Expr>) -> Option<Expr> {
  match exprs.len() {
    0 => None,
    1 => Some(exprs.swap_remove(0)),
    _ => Some(Expr::call("+", exprs)),
  }
}

// Assumes numer and denom are sorted according to `cmp_factor`.
// Searches for terms with a common base and combines them into the
// numerator.
fn move_common_terms_to_numer(numer: &mut [Factor], denom: &mut Vec<Factor>) {
  let mut i = 0;
  let mut j = 0;
  while i < numer.len() && j < denom.len() {
    match cmp_factor(&numer[i], &denom[j]) {
      Ordering::Less => {
        i += 1;
      }
      Ordering::Greater => {
        j += 1;
      }
      Ordering::Equal => {
        // We found two factors with the same base. Move them to the
        // numerator and group the exponents.
        let numer_exp = numer[i].exponent_mut();
        *numer_exp = Some(Expr::call("-", vec![
          numer_exp.clone().unwrap_or_else(Expr::one),
          denom[j].exponent_or_one(),
        ]));
        i += 1;
        denom.remove(j);
      }
    }
  }
}

fn cmp_factor(a: &Factor, b: &Factor) -> std::cmp::Ordering {
  cmp_expr(a.base(), b.base())
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
      Expr::call("*", vec![Expr::from(3), Expr::from(4), Expr::from(5)]),
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
      Expr::call("*", vec![Expr::from(2), Expr::from(3)]), // Note: The 1 is eliminated from the numerator.
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
    // Note: All '1's are eliminated.
    assert_eq!(new_expr, Expr::call("*", vec![
      Expr::call("*", vec![Expr::from(2), Expr::from(3)]),
      Expr::call("*", vec![var("x"), var("y")]),
    ]));
  }

  #[test]
  fn test_fraction_partial_split_evaluation() {
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
  fn test_fraction_partial_split_with_all_terms_scalar() {
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

  #[test]
  fn test_factor_sorter_on_product() {
    let expr = Expr::call("*", vec![
      Expr::call("^", vec![var("x"), Expr::from(2)]),
      var("y"),
      var("x"),
      var("y"),
      var("z"),
      Expr::call("^", vec![var("x"), Expr::from(2)]),
      Expr::call("^", vec![var("t"), Expr::from(1)]),
    ]);
    let (new_expr, errors) = run_simplifier(&FactorSorter::new(), expr);
    assert!(errors.is_empty());
    assert_eq!(new_expr, Expr::call("*", vec![
      var("t"),
      Expr::call("^", vec![var("x"), Expr::call("+", vec![Expr::from(2), Expr::from(1), Expr::from(2)])]),
      Expr::call("^", vec![var("y"), Expr::call("+", vec![Expr::from(1), Expr::from(1)])]),
      var("z"),
    ]));
  }

  #[test]
  fn test_factor_sorter_on_quotient_with_no_common_terms() {
    let expr = Expr::call("/", vec![
      Expr::call("*", vec![
        Expr::call("^", vec![var("x"), Expr::from(2)]),
        var("y"),
        var("z"),
      ]),
      Expr::call("*", vec![
        Expr::call("^", vec![var("a"), Expr::from(2)]),
        var("t"),
      ]),
    ]);
    let (new_expr, errors) = run_simplifier(&FactorSorter::new(), expr.clone());
    assert!(errors.is_empty());
    assert_eq!(new_expr, expr);
  }

  #[test]
  fn test_factor_sorter_on_quotient_with_common_terms() {
    let expr = Expr::call("/", vec![
      Expr::call("*", vec![
        Expr::call("^", vec![var("x"), Expr::from(2)]),
        var("z"),
        var("y"),
        var("a"),
      ]),
      Expr::call("*", vec![
        Expr::call("^", vec![var("z"), Expr::from(3)]),
        Expr::call("^", vec![var("y"), Expr::from(2)]),
        Expr::call("^", vec![var("x"), Expr::from(1)]),
        var("b"),
      ]),
    ]);
    let (new_expr, errors) = run_simplifier(&FactorSorter::new(), expr);
    assert!(errors.is_empty());
    assert_eq!(new_expr, Expr::call("/", vec![
      Expr::call("*", vec![
        var("a"),
        Expr::call("^", vec![var("x"), Expr::call("-", vec![Expr::from(2), Expr::from(1)])]),
        Expr::call("^", vec![var("y"), Expr::call("-", vec![Expr::from(1), Expr::from(2)])]),
        Expr::call("^", vec![var("z"), Expr::call("-", vec![Expr::from(1), Expr::from(3)])]),
      ]),
      var("b"),
    ]));
  }
}
