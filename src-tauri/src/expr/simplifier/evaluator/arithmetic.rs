
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
      builder::any_arity().of_type(ExprToNumber).and_then(|args, _| {
        let difference = args.into_iter().reduce(|a, b| a - b).unwrap_or(Number::zero());
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
      builder::any_arity().of_type(ExprToNumber).and_then(|args, _| {
        let quotient = args.into_iter().reduce(|a, b| a / b).unwrap_or(Number::one());
        Ok(Expr::from(quotient))
      })
    )
    .build()
}
