
//! Functions that represent concrete data in the language. These
//! functions will usually not simplify, but they might have some
//! useful properties we can exploit.

use crate::expr::Expr;
use crate::expr::function::Function;
use crate::expr::function::builder::{self, FunctionBuilder};
use crate::expr::function::table::FunctionTable;
use crate::expr::prisms;
use crate::util::prism::Identity;

pub fn append_datatype_functions(table: &mut FunctionTable) {
  table.insert(vector_function());
  table.insert(complex_function());
  table.insert(quaternion_function());
  table.insert(closed_interval());
  table.insert(right_open_interval());
  table.insert(left_open_interval());
  table.insert(full_open_interval());
  table.insert(incomplete_object());
}

pub fn vector_function() -> Function {
  FunctionBuilder::new("vector")
    .set_derivative(
      |args, engine| {
        let args = engine.differentiate_each(args)?;
        Ok(Expr::call("vector", args))
      }
    )
    .build()
}

pub fn complex_function() -> Function {
  FunctionBuilder::new("complex")
    .add_case(
      // Zero imaginary part
      builder::arity_two().of_types(Identity, prisms::ExprToZero).and_then(|a, _, _| {
        Ok(a)
      })
    )
    .set_derivative(
      |args, engine| {
        let args = engine.differentiate_each(args)?;
        Ok(Expr::call("complex", args))
      }
    )
    .build()
}

pub fn quaternion_function() -> Function {
  FunctionBuilder::new("quat")
    .add_case(
      // Only real part
      builder::arity_four().of_types(Identity, prisms::ExprToZero, prisms::ExprToZero, prisms::ExprToZero).and_then(|a, _, _, _, _| {
        Ok(a)
      })
    )
    .add_case(
      // Only complex part
      builder::arity_four().of_types(Identity, Identity, prisms::ExprToZero, prisms::ExprToZero).and_then(|a, b, _, _, _| {
        Ok(Expr::call("complex", vec![a, b]))
      })
    )
    .set_derivative(
      |args, engine| {
        let args = engine.differentiate_each(args)?;
        Ok(Expr::call("quat", args))
      }
    )
    .build()
}

pub fn closed_interval() -> Function {
  FunctionBuilder::new("..")
    .build()
}

pub fn right_open_interval() -> Function {
  FunctionBuilder::new("..^")
    .build()
}

pub fn left_open_interval() -> Function {
  FunctionBuilder::new("^..")
    .build()
}

pub fn full_open_interval() -> Function {
  FunctionBuilder::new("^..^")
    .build()
}

pub fn incomplete_object() -> Function {
  FunctionBuilder::new("incomplete")
    .build()
}
