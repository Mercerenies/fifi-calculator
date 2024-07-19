
use crate::expr::simplifier::{Simplifier, SimplifierContext};
use crate::expr::Expr;
use crate::units::parsing::UnitParser;
use crate::units::simplifier::simplify_compatible_units;
use crate::expr::number::Number;
use super::parser::parse_composite_unit_expr;
use super::output::tagged_into_expr_lossy;

use num::One;

/// Simplifier which cancels off compatible units in unit-like
/// expressions.
#[derive(Debug)]
pub struct UnitSimplifier<'a, P: ?Sized> {
  unit_parser: &'a P,
}

impl<'a, P> UnitSimplifier<'a, P>
where P: UnitParser<Number> + ?Sized {
  pub fn new(unit_parser: &'a P) -> Self {
    Self { unit_parser }
  }
}

impl<'a, P> Clone for UnitSimplifier<'a, P>
where P: ?Sized {
  fn clone(&self) -> Self {
    Self { unit_parser: self.unit_parser }
  }
}

impl<'a, P> Simplifier for UnitSimplifier<'a, P>
where P: UnitParser<Number> + ?Sized {
  fn simplify_expr_part(&self, expr: Expr, _: &mut SimplifierContext) -> Expr {
    dbg!(expr.to_string());
    let tagged = parse_composite_unit_expr(self.unit_parser, expr);
    if tagged.unit.is_one() {
      // No units, so nothing to simplify
      return tagged_into_expr_lossy(tagged);
    }
    let simplified_unit = simplify_compatible_units(tagged.unit.clone());
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
