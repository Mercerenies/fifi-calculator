
mod base;
pub mod chained;
pub mod evaluator;
pub mod error;
pub mod identity;

pub use base::Simplifier;
use evaluator::FunctionEvaluator;
use crate::expr::function::table::FunctionTable;

pub fn default_simplifier(function_table: &FunctionTable) -> Box<dyn Simplifier + '_> {
  Box::new(FunctionEvaluator::new(function_table))
}
