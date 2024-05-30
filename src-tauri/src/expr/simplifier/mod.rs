
mod base;
pub mod chained;
pub mod evaluator;
pub mod error;
pub mod identity;

pub use base::Simplifier;
use evaluator::FunctionEvaluator;
use evaluator::arithmetic::arithmetic_functions;
use evaluator::basic::basic_functions;

use std::collections::HashMap;

pub fn default_simplifier() -> Box<dyn Simplifier> {
  Box::new(function_evaluator())
}

fn function_evaluator() -> FunctionEvaluator {
  let mut map = HashMap::new();
  map.extend(basic_functions());
  map.extend(arithmetic_functions());
  FunctionEvaluator::from_hashmap(map)
}
