
//! Functions which operate on vectors.

use crate::expr::Expr;
use crate::expr::function::Function;
use crate::expr::function::table::FunctionTable;
use crate::expr::function::builder::{self, FunctionBuilder};
use crate::expr::vector::Vector;
use crate::expr::vector::tensor::Tensor;
use crate::expr::prisms;
use crate::expr::simplifier::error::SimplifierError;
use crate::util::repeated;
use crate::util::prism::Identity;

use std::cmp::Ordering;

pub fn append_tensor_functions(table: &mut FunctionTable) {
  table.insert(vconcat());
  table.insert(repeat());
  table.insert(iota());
  table.insert(head());
  table.insert(tail());
  table.insert(last());
  table.insert(init());
  table.insert(cons());
  table.insert(snoc());
}

fn is_empty_vector(expr: &Expr) -> bool {
  expr == &Expr::from(Vector::empty())
}

pub fn vconcat() -> Function {
  FunctionBuilder::new("vconcat")
    .permit_flattening()
    .set_identity(is_empty_vector)
    .add_case(
      // Vector concatenation
      builder::any_arity().of_type(prisms::ExprToTensor).and_then(|args, _| {
        let sum = args.into_iter()
          .map(Tensor::into_vector)
          .fold(Vector::empty(), Vector::append);
        Ok(sum.into())
      })
    )
    .add_case(
      // String concatenation
      builder::any_arity().of_type(prisms::expr_to_string()).and_then(|args, _| {
        Ok(Expr::from(args.join("")))
      })
    )
    .set_derivative(
      |args, engine| {
        // Pointwise derivative, similar to vectors.
        let args = engine.differentiate_each(args)?;
        Ok(Expr::call("vconcat", args))
      }
    )
    .build()
}

pub fn repeat() -> Function {
  FunctionBuilder::new("repeat")
    .add_case(
      builder::arity_two().of_types(Identity, prisms::expr_to_usize()).and_then(|value, len, _| {
        let vector: Vector = repeated(value, len);
        Ok(vector.into())
      })
    )
    .build()
}

pub fn iota() -> Function {
  FunctionBuilder::new("iota")
    .add_case(
      builder::arity_one().of_type(prisms::expr_to_i64()).and_then(|arg, _| {
        let vector: Vector = {
          match arg.cmp(&0) {
            Ordering::Greater => {
              (1..=arg).map(Expr::from).collect()
            }
            Ordering::Less => {
              (arg..=-1).map(Expr::from).collect()
            }
            Ordering::Equal => {
              Vector::empty()
            }
          }
        };
        Ok(vector.into())
      })
    )
    .build()
}

pub fn head() -> Function {
  FunctionBuilder::new("head")
    .add_case(
      builder::arity_one().of_type(prisms::ExprToVector).and_then(|vec, ctx| {
        if vec.is_empty() {
          ctx.errors.push(SimplifierError::custom_error("head", "head called on empty vector"));
          Err(vec)
        } else {
          let mut vec = Vec::from(vec);
          Ok(vec.swap_remove(0)) // bounds safety: We just checked if the vec was empty
        }
      })
    )
    .build()
}

pub fn tail() -> Function {
  FunctionBuilder::new("tail")
    .add_case(
      builder::arity_one().of_type(prisms::ExprToVector).and_then(|vec, ctx| {
        if vec.is_empty() {
          ctx.errors.push(SimplifierError::custom_error("tail", "tail called on empty vector"));
          Err(vec)
        } else {
          let mut vec = Vec::from(vec);
          vec.remove(0); // bounds safety: We just checked if the vec was empty
          Ok(Vector::from(vec).into())
        }
      })
    )
    .build()
}

pub fn last() -> Function {
  FunctionBuilder::new("last")
    .add_case(
      builder::arity_one().of_type(prisms::ExprToVector).and_then(|vec, ctx| {
        if vec.is_empty() {
          ctx.errors.push(SimplifierError::custom_error("last", "last called on empty vector"));
          Err(vec)
        } else {
          let mut vec = Vec::from(vec);
          Ok(vec.swap_remove(vec.len() - 1)) // bounds safety: We just checked if the vec was empty
        }
      })
    )
    .build()
}

pub fn init() -> Function {
  FunctionBuilder::new("init")
    .add_case(
      builder::arity_one().of_type(prisms::ExprToVector).and_then(|vec, ctx| {
        if vec.is_empty() {
          ctx.errors.push(SimplifierError::custom_error("init", "init called on empty vector"));
          Err(vec)
        } else {
          let mut vec = Vec::from(vec);
          vec.pop().unwrap(); // unwrap safety: We just checked if the vec was empty
          Ok(Vector::from(vec).into())
        }
      })
    )
    .build()
}

pub fn cons() -> Function {
  FunctionBuilder::new("cons")
    .add_case(
      builder::arity_two().of_types(Identity, prisms::ExprToVector).and_then(|new_value, mut vec, _| {
        vec.as_mut_vec().insert(0, new_value);
        Ok(vec.into())
      })
    )
    .build()
}

pub fn snoc() -> Function {
  FunctionBuilder::new("snoc")
    .add_case(
      builder::arity_two().of_types(prisms::ExprToVector, Identity).and_then(|mut vec, new_value, _| {
        vec.as_mut_vec().push(new_value);
        Ok(vec.into())
      })
    )
    .build()
}
