
mod base;
pub mod chained;
pub mod evaluator;
pub mod flattener;
pub mod error;
pub mod identity;

pub use base::{Simplifier, SequentialSimplifier};

use crate::expr::Expr;
use crate::expr::function::table::FunctionTable;
use crate::errorlist::ErrorList;
use error::SimplifierError;

#[derive(Debug)]
struct DefaultSimplifier<'a> {
  function_table: &'a FunctionTable,
}

// This could technically be a SequentialSimplifier, but those incur a
// lot of dynamic function calls (each individual element is a `dyn
// Simplifier`), and we call these things frequently. So it's more
// efficient to just hand-write the implementation we need.
impl<'a> Simplifier for DefaultSimplifier<'a> {
  fn simplify_expr_part(&self, mut expr: Expr, errors: &mut ErrorList<SimplifierError>) -> Expr {
    expr = evaluator::FunctionEvaluator::new(self.function_table).simplify_expr_part(expr, errors);
    expr = flattener::FunctionFlattener::new(self.function_table).simplify_expr_part(expr, errors);
    expr
  }
}

pub fn default_simplifier(function_table: &FunctionTable) -> Box<dyn Simplifier + '_> {
  Box::new(DefaultSimplifier { function_table })
}
