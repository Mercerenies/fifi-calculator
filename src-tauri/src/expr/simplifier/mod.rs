
mod base;
pub mod chained;
pub mod evaluator;
pub mod flattener;
pub mod error;
pub mod identity;
pub mod interval;
pub mod partial;
pub mod repeated;

pub use base::{Simplifier, SimplifierContext};

use crate::expr::Expr;
use crate::expr::function::table::FunctionTable;
use repeated::RepeatedSimplifier;

#[derive(Debug)]
struct DefaultSimplifier<'a> {
  function_table: &'a FunctionTable,
}

// This could technically be built up as a ChainedSimplifier, but
// those incur a lot of dynamic function calls (each individual
// element is a `dyn Simplifier`), and we call these things
// frequently. So it's more efficient to just hand-write the
// implementation we need.
impl<'a> Simplifier for DefaultSimplifier<'a> {
  fn simplify_expr_part(&self, mut expr: Expr, ctx: &mut SimplifierContext) -> Expr {
    expr = partial::IdentityRemover::new(self.function_table).simplify_expr_part(expr, ctx);
    expr = evaluator::FunctionEvaluator::new(self.function_table).simplify_expr_part(expr, ctx);
    expr = flattener::FunctionFlattener::new(self.function_table).simplify_expr_part(expr, ctx);
    expr = interval::IntervalNormalizer::new().simplify_expr_part(expr, ctx);
    expr
  }
}

pub fn default_simplifier(function_table: &FunctionTable) -> Box<dyn Simplifier + '_> {
  // We repeat the DefaultSimplifier pipeline a few times, to make
  // sure we get all reasonable simplifications. The choice of 5 times
  // is arbitrary.
  let default_simplifier = DefaultSimplifier { function_table };
  Box::new(RepeatedSimplifier::new(default_simplifier, 5))
}
