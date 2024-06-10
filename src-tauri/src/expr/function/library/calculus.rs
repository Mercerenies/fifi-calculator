
//! Functions for doing basic calculus.

use crate::util::prism::Identity;
use crate::expr::simplifier::error::SimplifierError;
use crate::expr::function::Function;
use crate::expr::function::table::FunctionTable;
use crate::expr::function::builder::{self, FunctionBuilder};
use crate::expr::prisms;
use crate::expr::calculus::differentiate;

pub fn append_calculus_functions(table: &mut FunctionTable) {
  table.insert(deriv());
}

pub fn deriv() -> Function {
  FunctionBuilder::new("deriv")
    .add_case(
      builder::arity_two().of_types(Identity::new(), prisms::ExprToVar).and_then(|expr, var, context| {
        match differentiate(context.function_table, expr, var.clone()) {
          Ok(expr) => Ok(expr),
          Err(failure) => {
            context.errors.push(SimplifierError::new("deriv", failure.error));
            Err((failure.original_expr, var))
          }
        }
      })
    )
    .build()
}
