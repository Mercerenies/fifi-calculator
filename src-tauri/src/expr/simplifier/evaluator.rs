
use crate::expr::Expr;
use crate::expr::number::Number;
use crate::errorlist::ErrorList;
use super::base::Simplifier;
use super::error::SimplifierError;

use num::Zero;

/// `FunctionEvaluator` is a [`Simplifier`] that evaluates known
/// functions when all of the arguments have known numerical values.
#[derive(Debug, Clone)]
pub struct FunctionEvaluator;

impl Simplifier for FunctionEvaluator {
  fn simplify_expr_part(&self, expr: Expr, _errors: &mut ErrorList<SimplifierError>) -> Expr {
    match expr {
      Expr::Call(function_name, args) => {
        let Some(arg_values) = args_try_into_numbers(args.clone()) else {
          return Expr::Call(function_name, args) // Pass through
        };
        match function_name.as_ref() {
          "+" => add(arg_values),
          _ => Expr::Call(function_name, args), // Pass through
        }
      }
      expr => {
        // Pass through
        expr
      }
    }
  }
}

fn args_try_into_numbers(args: Vec<Expr>) -> Option<Vec<Number>> {
  args.into_iter().map(|expr| expr.try_into()).collect::<Result<Vec<_>, _>>().ok()
}

fn add(exprs: Vec<Number>) -> Expr {
  let sum = exprs.into_iter().reduce(|a, b| a + b).unwrap_or(Number::zero());
  Expr::from(sum)
}
