
use crate::expr::Expr;
use crate::expr::prisms::expr_to_interval;
use crate::util::prism::Prism;
use super::base::{Simplifier, SimplifierContext};

/// A relatively simple [`Simplifier`] that normalizes the
/// representation of any
/// [`Interval`](crate::expr::interval::Interval) values. That is,
/// this simplifier applies to any applications of the `..`, `^..`,
/// `..^`, and `^..^` operators to two arguments, both of which are
/// real numbers.
#[derive(Debug, Default)]
pub struct IntervalNormalizer {
  _priv: (),
}

impl IntervalNormalizer {
  pub fn new() -> Self {
    Self::default()
  }
}

impl Simplifier for IntervalNormalizer {
  fn simplify_expr_part(&self, expr: Expr, _ctx: &mut SimplifierContext) -> Expr {
    match expr_to_interval().narrow_type(expr) {
      Err(expr) => expr,
      Ok(raw_interval) => Expr::from(raw_interval.normalize()),
    }
  }
}
