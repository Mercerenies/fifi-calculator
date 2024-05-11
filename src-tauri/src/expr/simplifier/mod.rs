
mod base;
pub mod chained;
pub mod evaluator;
pub mod error;
pub mod identity;

pub use base::Simplifier;

pub fn default_simplifier() -> Box<dyn Simplifier> {
  Box::new(evaluator::FunctionEvaluator)
}
