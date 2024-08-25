
use crate::expr::function::Function;
use crate::expr::function::builder::{self, FunctionBuilder};
use crate::expr::function::table::FunctionTable;
use crate::expr::predicates;
use crate::expr::prisms;
use crate::expr::number::ComplexLike;

use num::{Zero, One};

pub fn append_basic_functions(table: &mut FunctionTable) {
  table.insert(identity_function());
  table.insert(or_function());
  table.insert(and_function());
}

pub fn identity_function() -> Function {
  FunctionBuilder::new("identity")
    .add_case(
      builder::arity_one().and_then(|arg, _| Ok(arg))
    )
    .set_derivative(
      builder::arity_one_deriv("identity", |expr, engine| {
        engine.differentiate(expr)
      })
    )
    .build()
}

// TODO: Should work on strings and vectors of literals as well
pub fn or_function() -> Function {
  // Python-style "or" which returns the first argument which is not
  // zero.
  //
  // TODO: Consider allowing this to short-circuit somehow. Right now
  // it only simplifies if all quantities are known.
  FunctionBuilder::new("||")
    .add_partial_eval_rule(Box::new(predicates::is_complex))
    .add_case(
      builder::any_arity().of_type(prisms::ExprToComplex).and_then(|args, _| {
        let first_nonzero_value = args.into_iter()
          .fold(ComplexLike::zero(), |acc, arg| if acc.is_zero() { arg } else { acc });
        Ok(first_nonzero_value.into())
      })
    )
    .build()
}

// TODO: Should work on strings and vectors of literals as well
pub fn and_function() -> Function {
  // Python-style "and" which returns the first argument which is
  // considered to be zero.
  //
  // TODO: Consider allowing this to short-circuit somehow. Right now
  // it only simplifies if all quantities are known.
  FunctionBuilder::new("&&")
    .add_partial_eval_rule(Box::new(predicates::is_complex))
    .add_case(
      builder::any_arity().of_type(prisms::ExprToComplex).and_then(|args, _| {
        let first_zero_value = args.into_iter()
          .fold(ComplexLike::one(), |acc, arg| if acc.is_zero() { acc } else { arg });
        Ok(first_zero_value.into())
      })
    )
    .build()
}
