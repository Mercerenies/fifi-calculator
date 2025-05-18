
//! Evaluation rules for datetime-related functions.

use crate::expr::Expr;
use crate::expr::prisms::{expr_to_number, expr_to_datetime};
use crate::expr::function::Function;
use crate::expr::function::table::FunctionTable;
use crate::expr::function::builder::{self, FunctionBuilder};

pub fn append_datetime_functions(table: &mut FunctionTable) {
  table.insert(datetime_rel());
}

// TODO Technically this is differentiable in its first argument (but
// not its second; it's kind of like conj() if we had treated it as a
// two-arg function).
pub fn datetime_rel() -> Function {
  FunctionBuilder::new("datetime_rel")
    .add_case(
      // If given two datetime objects, equivalent to subtraction.
      builder::arity_two().both_of_type(expr_to_datetime()).and_then(|arg, rel, _| {
        Ok(Expr::call("-", vec![arg.into(), rel.into()]))
      })
    )
    .add_case(
      // If given a real number and a datetime, equivalent to
      // addition.
      builder::arity_two().of_types(expr_to_number(), expr_to_datetime()).and_then(|arg, rel, _| {
        Ok(Expr::call("+", vec![arg.into(), rel.into()]))
      })
    )
    .build()
}
