
//! Basic arithmetic function evaluation rules.

use super::function::Function;
use super::builder::{self, FunctionBuilder};
use crate::expr::prisms::ExprToNumber;
use crate::expr::Expr;
use crate::expr::number::Number;

use num::{Zero, One};

use std::collections::HashMap;

pub fn arithmetic_functions() -> HashMap<String, Function> {
  let mut functions = HashMap::new();
  functions.insert("+".to_string(), addition());
  functions.insert("-".to_string(), subtraction());
  functions.insert("*".to_string(), multiplication());
  functions.insert("/".to_string(), division());
  functions.insert("%".to_string(), modulo());
  functions.insert("\\".to_string(), floor_division());
  functions
}

pub fn addition() -> Function {
  FunctionBuilder::new("+")
    .add_case(
      builder::any_arity().of_type(ExprToNumber).and_then(|args, _| {
        let sum = args.into_iter().reduce(|a, b| a + b).unwrap_or(Number::zero());
        Ok(Expr::from(sum))
      })
    )
    .build()
}

pub fn subtraction() -> Function {
  FunctionBuilder::new("-")
    .add_case(
      builder::arity_two().both_of_type(ExprToNumber).and_then(|arg1, arg2, _| {
        let difference = arg1 - arg2;
        Ok(Expr::from(difference))
      })
    )
    .build()
}

pub fn multiplication() -> Function {
  FunctionBuilder::new("*")
    .add_case(
      builder::any_arity().of_type(ExprToNumber).and_then(|args, _| {
        let product = args.into_iter().reduce(|a, b| a * b).unwrap_or(Number::one());
        Ok(Expr::from(product))
      })
    )
    .build()
}

pub fn division() -> Function {
  FunctionBuilder::new("/")
    .add_case(
      builder::arity_two().both_of_type(ExprToNumber).and_then(|arg1, arg2, _| {
        let quotient = arg1 / arg2;
        Ok(Expr::from(quotient))
      })
    )
    .build()
}

pub fn modulo() -> Function {
  FunctionBuilder::new("%")
    .add_case(
      builder::arity_two().both_of_type(ExprToNumber).and_then(|arg1, arg2, _| {
        Ok(Expr::from(arg1 % arg2))
      })
    )
    .build()
}

pub fn floor_division() -> Function {
  FunctionBuilder::new("\\")
    .add_case(
      builder::arity_two().both_of_type(ExprToNumber).and_then(|arg1, arg2, _| {
        Ok(Expr::from(arg1 % arg2))
      })
    )
    .build()
}
