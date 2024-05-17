
pub mod arithmetic;
pub mod builder;
pub mod function;

use crate::expr::Expr;
use crate::errorlist::ErrorList;
use super::base::Simplifier;
use super::error::SimplifierError;
use function::Function;

use std::collections::HashMap;

/// `FunctionEvaluator` is a [`Simplifier`] that evaluates known
/// functions when all of the arguments have known numerical values.
#[derive(Debug, Default)]
pub struct FunctionEvaluator {
  known_functions: HashMap<String, Function>,
}

impl FunctionEvaluator {
  pub fn new() -> Self {
    Self::default()
  }

  pub fn from_hashmap(known_functions: HashMap<String, Function>) -> Self {
    Self { known_functions }
  }

  pub fn insert(&mut self, function: Function) -> Option<Function> {
    self.known_functions.insert(function.name().to_owned(), function)
  }

  pub fn get(&self, name: &str) -> Option<&Function> {
    self.known_functions.get(name)
  }
}

impl Simplifier for FunctionEvaluator {
  fn simplify_expr_part(&self, expr: Expr, errors: &mut ErrorList<SimplifierError>) -> Expr {
    match expr {
      Expr::Call(function_name, args) => {
        let Some(known_function) = self.get(&function_name) else {
          return Expr::Call(function_name, args);
        };
        match known_function.call(args, errors) {
          Ok(expr) => expr,
          Err(args) => Expr::Call(function_name, args),
        }
      }
      expr => {
        // Pass through
        expr
      }
    }
  }
}
