
use crate::expr::Expr;
use crate::expr::function::table::FunctionTable;
use crate::errorlist::ErrorList;
use super::base::Simplifier;
use super::error::SimplifierError;

/// `FunctionEvaluator` is a [`Simplifier`] that evaluates known
/// functions when all of the arguments have known acceptable values.
/// Usually, this means all arguments are numerical ground terms, with
/// no variables.
#[derive(Debug)]
pub struct FunctionEvaluator<'a> {
  function_table: &'a FunctionTable,
}

impl<'a> FunctionEvaluator<'a> {
  pub fn new(function_table: &'a FunctionTable) -> Self {
    Self { function_table }
  }
}

impl<'a> Simplifier for FunctionEvaluator<'a> {
  fn simplify_expr_part(&self, expr: Expr, errors: &mut ErrorList<SimplifierError>) -> Expr {
    match expr {
      Expr::Call(function_name, args) => {
        let Some(known_function) = self.function_table.get(&function_name) else {
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
