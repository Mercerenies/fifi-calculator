
mod base;
pub mod chained;
pub mod dollar_sign;
pub mod evaluator;
pub mod flattener;
pub mod error;
pub mod identity;
pub mod interval;
pub mod partial;
pub mod repeated;
pub mod term;
pub mod unicode;

pub use base::{Simplifier, SimplifierContext};

use crate::expr::Expr;
use crate::expr::function::table::FunctionTable;
use crate::expr::function::distributive::{DistributiveRuleSimplifier, DistributiveRuleset};
use repeated::RepeatedSimplifier;
use unicode::UnicodeSimplifier;

struct DefaultSimplifier<'a> {
  function_table: &'a FunctionTable,
  // We store these in advance since they're nontrivial to construct.
  // The others all have trivial constructors, so we create them
  // during `simplify_expr_part`'s body.
  unicode_simplifier: UnicodeSimplifier,
  distributive_rule_simplifier: DistributiveRuleSimplifier,
}

// This could technically be built up as a ChainedSimplifier, but
// those incur a lot of dynamic function calls (each individual
// element is a `dyn Simplifier`), and we call these things
// frequently. So it's more efficient to just hand-write the
// implementation we need.
impl<'a> Simplifier for DefaultSimplifier<'a> {
  fn simplify_expr_part(&self, mut expr: Expr, ctx: &mut SimplifierContext) -> Expr {
    expr = self.unicode_simplifier.simplify_expr_part(expr, ctx);
    expr = partial::IdentityRemover::new(self.function_table).simplify_expr_part(expr, ctx);
    expr = flattener::FunctionFlattener::new(self.function_table).simplify_expr_part(expr, ctx);
    expr = self.distributive_rule_simplifier.simplify_expr_part(expr, ctx);
    expr = term::TermPartialSplitter::new().simplify_expr_part(expr, ctx);
    expr = evaluator::FunctionEvaluator::new(self.function_table).simplify_expr_part(expr, ctx);
    expr = interval::IntervalNormalizer::new().simplify_expr_part(expr, ctx);
    expr
  }
}

pub fn default_simplifier(function_table: &FunctionTable) -> Box<dyn Simplifier + '_> {
  // We repeat the DefaultSimplifier pipeline a few times, to make
  // sure we get all reasonable simplifications. The choice of 5 times
  // is arbitrary.
  let default_simplifier = DefaultSimplifier {
    function_table,
    unicode_simplifier: UnicodeSimplifier::from_common_aliases(),
    distributive_rule_simplifier: DistributiveRuleSimplifier::new(DistributiveRuleset::from_common_rules()),
  };
  Box::new(RepeatedSimplifier::new(default_simplifier, 5))
}

#[cfg(test)]
pub(crate) mod test_utils {
  use super::*;
  use super::base::SimplifierContext;
  use super::error::SimplifierError;
  use crate::mode::calculation::CalculationMode;
  use crate::errorlist::ErrorList;

  pub fn run_simplifier(simplifier: &impl Simplifier, expr: Expr) -> (Expr, ErrorList<SimplifierError>) {
    let mut errors = ErrorList::new();
    let mut context = SimplifierContext {
      base_simplifier: simplifier,
      calculation_mode: CalculationMode::default(),
      errors: &mut errors,
    };
    let expr = simplifier.simplify_expr(expr, &mut context);
    (expr, errors)
  }
}
