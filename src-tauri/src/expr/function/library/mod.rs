
//! Library of built-in well-known mathematical functions and their
//! properties.
//!
//! This module only exports one function: [`build_function_table`].

use crate::expr::function::table::FunctionTable;

mod arithmetic;
mod basic;
mod calculus;
mod datatypes;
mod formula;
mod graphics;
mod symbolic;
mod transcendental;
mod tensor;

pub fn build_function_table() -> FunctionTable {
  let mut table = FunctionTable::new();
  arithmetic::append_arithmetic_functions(&mut table);
  basic::append_basic_functions(&mut table);
  calculus::append_calculus_functions(&mut table);
  datatypes::append_datatype_functions(&mut table);
  formula::append_formula_functions(&mut table);
  graphics::append_graphics_functions(&mut table);
  symbolic::append_symbolic_functions(&mut table);
  transcendental::append_transcendental_functions(&mut table);
  tensor::append_tensor_functions(&mut table);
  table
}
