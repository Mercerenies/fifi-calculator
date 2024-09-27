
use crate::expr::simplifier::{Simplifier, SimplifierContext};
use crate::expr::Expr;
use crate::expr::algebra::term::{Term, Sign, SignedTerm};
use crate::expr::algebra::polynomial::{Polynomial, parse_polynomial};
use crate::units::CompositeUnit;
use crate::units::tagged::Tagged;
use crate::units::parsing::UnitParser;
use crate::units::simplifier::{simplify_compatible_units, is_minimal};
use crate::expr::number::Number;
use super::parser::{parse_composite_unit_expr, parse_composite_unit_term};
use super::output::{tagged_into_expr_lossy, unit_into_term_lossy};

use num::One;
use itertools::Itertools;

/// Simplifier which cancels off compatible units in unit-like
/// expressions.
#[derive(Debug)]
pub struct UnitTermSimplifier<'a, P: ?Sized> {
  unit_parser: &'a P,
}

/// Simplifier which combines terms of the same dimension in a
/// summation.
#[derive(Debug)]
pub struct UnitPolynomialSimplifier<'a, P: ?Sized> {
  unit_parser: &'a P,
}

impl<'a, P> UnitTermSimplifier<'a, P>
where P: UnitParser<Number> + ?Sized {
  pub fn new(unit_parser: &'a P) -> Self {
    Self { unit_parser }
  }
}

impl<'a, P> UnitPolynomialSimplifier<'a, P>
where P: UnitParser<Number> + ?Sized {
  pub fn new(unit_parser: &'a P) -> Self {
    Self { unit_parser }
  }
}

impl<'a, P> Clone for UnitTermSimplifier<'a, P>
where P: ?Sized {
  fn clone(&self) -> Self {
    Self { unit_parser: self.unit_parser }
  }
}

impl<'a, P> Clone for UnitPolynomialSimplifier<'a, P>
where P: ?Sized {
  fn clone(&self) -> Self {
    Self { unit_parser: self.unit_parser }
  }
}

impl<'a, P> Simplifier for UnitTermSimplifier<'a, P>
where P: UnitParser<Number> + ?Sized {
  fn simplify_expr_part(&self, expr: Expr, _: &mut SimplifierContext) -> Expr {
    let tagged = parse_composite_unit_expr(self.unit_parser, expr);
    if tagged.unit.is_one() {
      // No units, so nothing to simplify
      return tagged_into_expr_lossy(tagged);
    }
    let simplified_unit = run_simplifications(tagged.unit.clone());
    // convert_or_panic: simplify_compatible_unit always retains the
    // dimension of its input.
    let tagged =
      if simplified_unit == tagged.unit {
        // Don't add a bunch of "* 1" nonsense if we're not actually
        // converting anything.
        tagged
      } else {
        tagged.convert_or_panic(simplified_unit)
      };
    tagged_into_expr_lossy(tagged)
  }
}

impl<'a, P> Simplifier for UnitPolynomialSimplifier<'a, P>
where P: UnitParser<Number> + ?Sized {
  fn simplify_expr_part(&self, expr: Expr, _: &mut SimplifierContext) -> Expr {
    let polynomial = parse_polynomial(expr);
    if polynomial.len() == 1 {
      return polynomial.into();
    }
    let grouped_terms = polynomial
      .into_terms()
      .into_iter()
      .map(|signed_term| {
        (signed_term.sign, parse_composite_unit_term(self.unit_parser, signed_term.term))
      })
      .into_group_map_by(|(_sign, tagged)| tagged.unit.dimension());
    // TODO Once we decide how we're sorting polynomials, do it here
    // too for consistency.
    let polynomial_terms = grouped_terms.into_values()
      .map(|terms| {
        let tagged_polynomial = simplify_sum(terms);
        SignedTerm::new(
          Sign::Positive,
          Term::singleton(tagged_polynomial.value.into()) * unit_into_term_lossy(tagged_polynomial.unit),
        )
      });
    Polynomial::new(polynomial_terms).into()
  }
}

fn simplify_sum(terms: Vec<(Sign, Tagged<Term, Number>)>) -> Tagged<Polynomial, Number> {
  assert!(!terms.is_empty(), "simplify_sum expected non-empty vector");
  let final_unit = terms[0].1.unit.clone();
  let final_terms = terms.into_iter().map(|(sign, term)| {
    let term =
      if final_unit == term.unit {
        term.value
      } else {
        // convert_or_panic: Should already be grouped by dimension,
        // per UnitPolynomialSimplifier.
        term.convert_or_panic(final_unit.clone()).value
      };
    SignedTerm::new(sign, term)
  });
  Tagged::new(
    Polynomial::new(final_terms),
    final_unit,
  )
}

fn run_simplifications(unit: CompositeUnit<Number>) -> CompositeUnit<Number> {
  // First, try simplifying units as-is (a simpler operation that is
  // also less destructive). If, after trying that, there are still
  // non-trivial dimensions in both the numerator and denominator,
  // then break apart composite units and try again.
  let unit = simplify_compatible_units(unit);
  if is_minimal(&unit) {
    return unit;
  }
  let unit = unit.expand_compositions();
  simplify_compatible_units(unit)
}
