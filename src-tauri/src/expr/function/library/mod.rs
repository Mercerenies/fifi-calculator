
//! Library of built-in well-known mathematical functions and their
//! properties.
//!
//! This module only exports one function: [`build_function_table`].

use super::table::FunctionTable;

mod arithmetic;
mod basic;
mod calculus;
mod complex;
mod datatypes;
mod datetime;
mod formula;
mod functor;
mod graphics;
mod statistics;
mod string;
mod symbolic;
mod transcendental;
mod tensor;

pub fn build_function_table() -> FunctionTable {
  let mut table = FunctionTable::new();
  arithmetic::append_arithmetic_functions(&mut table);
  basic::append_basic_functions(&mut table);
  calculus::append_calculus_functions(&mut table);
  complex::append_complex_functions(&mut table);
  datatypes::append_datatype_functions(&mut table);
  datetime::append_datetime_functions(&mut table);
  formula::append_formula_functions(&mut table);
  functor::append_functor_functions(&mut table);
  graphics::append_graphics_functions(&mut table);
  statistics::append_statistics_functions(&mut table);
  string::append_string_functions(&mut table);
  symbolic::append_symbolic_functions(&mut table);
  transcendental::append_transcendental_functions(&mut table);
  tensor::append_tensor_functions(&mut table);
  table
}
