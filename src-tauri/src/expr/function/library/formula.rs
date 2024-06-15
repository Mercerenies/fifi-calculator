
//! Functions that behave like mathematical relations to construct
//! top-level equations or inequalities.

use crate::expr::Expr;
use crate::expr::function::Function;
use crate::expr::function::builder::{self, FunctionBuilder};
use crate::expr::function::table::FunctionTable;
use crate::expr::prisms;

pub fn append_formula_functions(table: &mut FunctionTable) {
  // TODO: Add some functions
}

pub fn equal_to() -> Function {
  FunctionBuilder::new("=")
    .set_derivative( // TODO: Generalize this "pointwise derivative" pattern; make it a part of builder api
      |args, engine| {
        // Pointwise derivative
        let args = engine.differentiate_each(args)?;
        Ok(Expr::call("=", args))
      }
    )
    .build()
}

pub fn not_equal_to() -> Function {
  FunctionBuilder::new("!=")
    .set_derivative(
      |args, engine| {
        // Pointwise derivative
        let args = engine.differentiate_each(args)?;
        Ok(Expr::call("!=", args))
      }
    )
    .build()
}

pub fn less_than() -> Function {
  FunctionBuilder::new("<")
    .add_case(
      // Real number comparison
      builder::arity_two().both_of_type(prisms::ExprToNumber).and_then(|left, right, _| {
        Ok(Expr::from(left < right))
      })
    )
    .set_derivative(
      |args, engine| {
        // Pointwise derivative
        let args = engine.differentiate_each(args)?;
        Ok(Expr::call("<", args))
      }
    )
    .build()
}

pub fn less_than_or_equal() -> Function {
  FunctionBuilder::new("<=")
    .add_case(
      // Real number comparison
      builder::arity_two().both_of_type(prisms::ExprToNumber).and_then(|left, right, _| {
        Ok(Expr::from(left <= right))
      })
    )
    .set_derivative(
      |args, engine| {
        // Pointwise derivative
        let args = engine.differentiate_each(args)?;
        Ok(Expr::call("<=", args))
      }
    )
    .build()
}

pub fn greater_than() -> Function {
  FunctionBuilder::new(">")
    .add_case(
      // Real number comparison
      builder::arity_two().both_of_type(prisms::ExprToNumber).and_then(|left, right, _| {
        Ok(Expr::from(left > right))
      })
    )
    .set_derivative(
      |args, engine| {
        // Pointwise derivative
        let args = engine.differentiate_each(args)?;
        Ok(Expr::call(">", args))
      }
    )
    .build()
}

pub fn greater_than_or_equal() -> Function {
  FunctionBuilder::new(">=")
    .add_case(
      // Real number comparison
      builder::arity_two().both_of_type(prisms::ExprToNumber).and_then(|left, right, _| {
        Ok(Expr::from(left >= right))
      })
    )
    .set_derivative(
      |args, engine| {
        // Pointwise derivative
        let args = engine.differentiate_each(args)?;
        Ok(Expr::call(">=", args))
      }
    )
    .build()
}
