
//! Functions for doing basic calculus.

use crate::util::prism::Identity;
use crate::expr::Expr;
use crate::expr::var::Var;
use crate::expr::number::Number;
use crate::expr::simplifier::error::SimplifierError;
use crate::expr::function::{Function, FunctionContext};
use crate::expr::function::table::FunctionTable;
use crate::expr::function::builder::{self, FunctionBuilder};
use crate::expr::prisms::{ExprToVar, ExprToNumber};
use crate::expr::calculus::differentiate;

use num::{BigInt, ToPrimitive};

use std::convert::TryFrom;

pub fn append_calculus_functions(table: &mut FunctionTable) {
  table.insert(deriv());
}

pub fn deriv() -> Function {
  FunctionBuilder::new("deriv")
    .add_case(
      builder::arity_two().of_types(Identity::new(), ExprToVar).and_then(|expr, var, context| {
        nth_derivative(expr, var, 1, context).map_err(|(expr, var, _)| (expr, var))
      })
    )
    .add_case(
      builder::arity_three().of_types(Identity::new(), ExprToVar, ExprToNumber).and_then(|expr, var, n, context| {
        let n: BigInt = match BigInt::try_from(n) {
          Ok(n) => n,
          Err(err) => {
            context.errors.push(SimplifierError::new("deriv", err.clone()));
            return Err((expr, var, err.number));
          }
        };
        let Some(n) = n.to_usize() else {
          context.errors.push(SimplifierError::custom_error("deriv", "n argument too large"));
          return Err((expr, var, Number::from(n)));
        };
        nth_derivative(expr, var, n, context)
      })
    )
    .build()
}

fn nth_derivative(mut expr: Expr, var: Var, n: usize, context: &mut FunctionContext) -> Result<Expr, (Expr, Var, Number)> {
  for _ in 0..n {
    match differentiate(context.function_table, expr, var.clone()) {
      Ok(dexpr) => {
        expr = dexpr;
      }
      Err(failure) => {
        context.errors.push(SimplifierError::new("deriv", failure.error));
        return Err((failure.original_expr, var, Number::from(n)))
      }
    }
  }
  Ok(expr)
}
