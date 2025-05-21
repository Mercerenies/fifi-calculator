
//! Functions relating to functors (call expressions) as an abstract
//! concept.

use crate::util::prism::ErrorWithPayload;
use crate::expr::Expr;
use crate::expr::simplifier::error::SimplifierError;
use crate::expr::var::Var;
use crate::expr::vector::Vector;
use crate::expr::function::Function;
use crate::expr::function::builder::{self, FunctionBuilder};
use crate::expr::function::table::FunctionTable;
use crate::expr::prisms;

use thiserror::Error;

#[derive(Debug, Clone, Error)]
#[error("Functor index out of bounds")]
struct FunctorIndexOutOfBounds;

pub fn append_functor_functions(table: &mut FunctionTable) {
  table.insert(functor_head());
  table.insert(functor_args());
  table.insert(functor_nth_arg());
}

pub fn functor_head() -> Function {
  FunctionBuilder::new("fhead")
    .add_case(
      // Head of functor
      builder::arity_one().of_type(prisms::expr_to_functor_call()).and_then(|call, _| {
        // Return it as a variable if possible, or a string if it's an
        // exotic name.
        match Var::try_from(call.name) {
          Ok(v) => Ok(Expr::from(v)),
          Err(err) => {
            let call_name = err.recover_payload();
            Ok(Expr::from(call_name))
          }
        }
      })
    )
    .build()
}

pub fn functor_args() -> Function {
  FunctionBuilder::new("fargs")
    .add_case(
      // Arguments of functor (as a vector)
      builder::arity_one().of_type(prisms::expr_to_functor_call()).and_then(|call, _| {
        let args = Vector::from(call.args);
        Ok(args.into())
      })
    )
    .build()
}

pub fn functor_nth_arg() -> Function {
  FunctionBuilder::new("farg")
    .add_case(
      // Argument of functor (as a vector)
      builder::arity_two().of_types(prisms::expr_to_functor_call(), prisms::expr_to_i64())
        .and_then(|mut call, idx, ctx| {
          let Ok(idx) = usize::try_from(idx) else {
            ctx.errors.push(SimplifierError::new("farg", FunctorIndexOutOfBounds));
            return Err((call, idx));
          };
          if !(0..call.args.len()).contains(&idx) {
            ctx.errors.push(SimplifierError::new("farg", FunctorIndexOutOfBounds));
            return Err((call, idx as i64)); // `as i64` is safe because we just came from `i64`.
          }
          let arg = call.args.swap_remove(idx);
          Ok(arg)
        })
    )
    .build()
}
