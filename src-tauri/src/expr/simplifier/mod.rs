
mod base;
pub mod chained;
pub mod evaluator;
pub mod error;
pub mod identity;

pub use base::Simplifier;
use evaluator::FunctionEvaluator;
use evaluator::arithmetic::arithmetic_functions;

pub fn default_simplifier() -> Box<dyn Simplifier> {
  Box::new(function_evaluator())
}

fn function_evaluator() -> FunctionEvaluator {
  FunctionEvaluator::from_hashmap(arithmetic_functions())
}
