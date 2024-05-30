
//! Very simple function evaluation rules.

use super::function::Function;
use super::builder::{self, FunctionBuilder};

use std::collections::HashMap;

pub fn basic_functions() -> HashMap<String, Function> {
  let mut functions = HashMap::new();
  functions.insert("identity".to_string(), identity_function());
  functions
}

pub fn identity_function() -> Function {
  FunctionBuilder::new("identity")
    .add_case(
      builder::arity_one().and_then(|arg, _| Ok(arg))
    )
    .build()
}
