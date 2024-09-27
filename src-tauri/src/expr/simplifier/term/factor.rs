
use crate::expr::algebra::term::Term;
use crate::expr::algebra::factor::Factor;
use crate::expr::arithmetic::ArithExpr;
use crate::expr::Expr;
use crate::expr::atom::Atom;
use crate::expr::number::Number;
use crate::expr::ordering::cmp_expr;
use crate::expr::simplifier::base::{Simplifier, SimplifierContext};
use crate::util::{retain_into, insert_sorted_by, Recip};

use itertools::Itertools;
use num::Zero;

use std::cmp::Ordering;

/// `FactorSorter` is a [`Simplifier`] that orders the factors in the
/// numerator and denominator of a term according to a sensible and
/// canonical ordering. This simplifier also groups factors with the
/// same exponential base.
#[derive(Debug, Default)]
pub struct FactorSorter {
  _priv: (),
}

impl FactorSorter {
  pub fn new() -> Self {
    Self { _priv: () }
  }
}

impl Simplifier for FactorSorter {
  fn simplify_expr_part(&self, expr: Expr, _ctx: &mut SimplifierContext) -> Expr {
    let term = Term::parse(expr);
    let (numer, denom) = term.into_parts();
    let mut numer = group_and_sort_factors(numer);
    let mut denom = group_and_sort_factors(denom);
    move_common_terms_to_numer(&mut numer, &mut denom);
    flip_negative_terms_to(&mut denom, &mut numer);
    // Do not create a denominator if there isn't already one.
    if !denom.is_empty() {
      flip_negative_terms_to(&mut numer, &mut denom);
    }
    let numer = numer.into_iter().map(Factor::simplify_trivial_powers);
    let denom = denom.into_iter().map(Factor::simplify_trivial_powers);
    Term::from_parts(numer, denom).into()
  }
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
      Factor::from_parts(base, Some(ArithExpr::sum(exponents)))
        .simplify_trivial_powers()
    })
    .collect();
  factors.sort_by(cmp_factor);
  factors
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
        *numer_exp = Some(
          (ArithExpr::from(numer_exp.clone().unwrap_or_else(Expr::one)) -
           ArithExpr::from(denom[j].exponent_or_one())).into()
        );
        i += 1;
        denom.remove(j);
      }
    }
  }
}

fn cmp_factor(a: &Factor, b: &Factor) -> std::cmp::Ordering {
  cmp_expr(a.base(), b.base())
}

/// Assuming that both `source` and `dest` are sorted according to
/// [`cmp_factor`], this function moves all terms from `source` whose
/// exponents are nontrivial and satisfy [`is_obviously_negative`]
/// into `dest`, at the appropriate position to keep the terms in
/// `dest` sorted. All moved terms are reciprocated via the [`Recip`]
/// implementation on `Factor`.
///
/// Runs in `O(N + KlogM)`, where `N` is the length of `source`, `K`
/// is the number of `is_obviously_negative` exponents in `source`,
/// and `M` is the length of `dest`.
fn flip_negative_terms_to(source: &mut Vec<Factor>, dest: &mut Vec<Factor>) {
  let negative_factors = retain_into(source, |factor| {
    factor.exponent().map_or(true, |e| !is_obviously_negative(e))
  });
  for factor in negative_factors {
    insert_sorted_by(dest, factor.recip(), cmp_factor);
  }
}

/// Returns true if the expression appears negative to a casual
/// viewer. An expression appears negative if it is literally a
/// negative real-numbered constant, or if it is a unary application
/// of the `negate` function.
fn is_obviously_negative(expr: &Expr) -> bool {
  match expr {
    Expr::Atom(Atom::Number(n)) => *n < Number::zero(),
    Expr::Atom(_) => false,
    Expr::Call(f, args) => f == "negate" && args.len() == 1,
  }
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
      Expr::call("^", vec![var("x"), Expr::from(5)]),
      Expr::call("^", vec![var("y"), Expr::from(2)]),
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
        var("x")
      ]),
      Expr::call("*", vec![
        var("b"),
        var("y"),
        Expr::call("^", vec![var("z"), Expr::from(2)]),
      ]),
    ]));
  }

  #[test]
  fn test_factor_sorter_with_some_negatives() {
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
        Expr::call("^", vec![var("x"), Expr::from(3)]),
        var("b"),
      ]),
    ]);
    let (new_expr, errors) = run_simplifier(&FactorSorter::new(), expr);
    assert!(errors.is_empty());
    assert_eq!(new_expr, Expr::call("/", vec![
      var("a"),
      Expr::call("*", vec![
        var("b"),
        var("x"),
        var("y"),
        Expr::call("^", vec![var("z"), Expr::from(2)]),
      ]),
    ]));
  }
}
