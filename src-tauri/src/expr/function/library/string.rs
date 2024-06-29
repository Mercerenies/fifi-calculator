

//! Evaluation rules for transcendental and trigonometric functions.

use crate::expr::Expr;
use crate::expr::function::Function;
use crate::expr::function::table::FunctionTable;
use crate::expr::function::builder::{self, FunctionBuilder};
use crate::expr::prisms::expr_to_string;

pub fn append_string_functions(table: &mut FunctionTable) {
  table.insert(to_lowercase());
  table.insert(to_uppercase());
}

pub fn to_lowercase() -> Function {
  FunctionBuilder::new("lowercase")
    .add_case(
      builder::arity_one().of_type(expr_to_string()).and_then(|arg, _| {
        Ok(Expr::from(arg.to_lowercase()))
      })
    )
    .build()
}

pub fn to_uppercase() -> Function {
  FunctionBuilder::new("uppercase")
    .add_case(
      builder::arity_one().of_type(expr_to_string()).and_then(|arg, _| {
        Ok(Expr::from(arg.to_uppercase()))
      })
    )
    .build()
}
