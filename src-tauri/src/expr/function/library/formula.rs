
//! Functions that behave like mathematical relations to construct
//! top-level equations or inequalities.

use crate::expr::Expr;
use crate::expr::function::Function;
use crate::expr::function::builder::{self, FunctionBuilder};
use crate::expr::function::table::FunctionTable;
use crate::expr::prisms;

pub fn append_formula_functions(table: &mut FunctionTable) {
  table.insert(equal_to());
  table.insert(not_equal_to());
  table.insert(less_than());
  table.insert(greater_than());
  table.insert(less_than_or_equal());
  table.insert(greater_than_or_equal());
}

pub fn equal_to() -> Function {
  FunctionBuilder::new("=")
    .add_case(
      // Literal value comparison
      builder::arity_two().both_of_type(prisms::expr_to_literal()).and_then(|left, right, _| {
        Ok(Expr::from(left == right))
      })
    )
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
    .add_case(
      // Literal value comparison
      builder::arity_two().both_of_type(prisms::expr_to_literal()).and_then(|left, right, _| {
        Ok(Expr::from(left != right))
      })
    )
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
      // Real number (possibly infinite) comparison
      builder::arity_two().both_of_type(prisms::expr_to_unbounded_number()).and_then(|left, right, _| {
        Ok(Expr::from(left < right))
      })
    )
    .add_case(
      // String comparison
      builder::arity_two().both_of_type(prisms::expr_to_string()).and_then(|left, right, _| {
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
      // Real number (possibly infinite) comparison
      builder::arity_two().both_of_type(prisms::expr_to_unbounded_number()).and_then(|left, right, _| {
        Ok(Expr::from(left <= right))
      })
    )
    .add_case(
      // String comparison
      builder::arity_two().both_of_type(prisms::expr_to_string()).and_then(|left, right, _| {
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
      // Real number (possibly infinite) comparison
      builder::arity_two().both_of_type(prisms::expr_to_unbounded_number()).and_then(|left, right, _| {
        Ok(Expr::from(left > right))
      })
    )
    .add_case(
      // String comparison
      builder::arity_two().both_of_type(prisms::expr_to_string()).and_then(|left, right, _| {
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
      // Real number (possibly infinite) comparison
      builder::arity_two().both_of_type(prisms::expr_to_unbounded_number()).and_then(|left, right, _| {
        Ok(Expr::from(left >= right))
      })
    )
    .add_case(
      // String comparison
      builder::arity_two().both_of_type(prisms::expr_to_string()).and_then(|left, right, _| {
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
