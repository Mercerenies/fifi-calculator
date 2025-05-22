
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
  table.insert(functor_compile());
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
          let Ok(idx) = get_actual_index(call.args.len(), idx) else {
            ctx.errors.push(SimplifierError::new("farg", FunctorIndexOutOfBounds));
            return Err((call, idx));
          };
          let arg = call.args.swap_remove(idx);
          Ok(arg)
        })
    )
    .build()
}

pub fn functor_compile() -> Function {
  FunctionBuilder::new("fcompile")
    .add_case(
      // Head and vector of args
      builder::arity_two().of_types(prisms::expr_to_loose_var(), prisms::ExprToVector)
        .and_then(|head, args, _| {
          Ok(Expr::Call(head.into_name(), args.into()))
        })
    )
    .build()
}

/// Adjusts index to refer to an actual unsigned position in the
/// functor. If the index is negative, the length is added to it. If
/// the index is out of bounds of the functor (on either side) then an
/// error is returned.
fn get_actual_index(len: usize, mut signed_idx: i64) -> Result<usize, FunctorIndexOutOfBounds> {
  if signed_idx < 0 {
    signed_idx += len as i64;
  }
  let unsigned_idx = usize::try_from(signed_idx).map_err(|_| FunctorIndexOutOfBounds)?;
  if unsigned_idx < len {
    Ok(unsigned_idx)
  } else {
    Err(FunctorIndexOutOfBounds)
  }
}
