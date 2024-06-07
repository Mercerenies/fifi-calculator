
//! Library of built-in well-known mathematical functions and their
//! properties.
//!
//! This module only exports one function: [`build_function_table`].

use crate::expr::function::table::FunctionTable;

mod arithmetic;
mod basic;

pub fn build_function_table() -> FunctionTable {
  let mut table = FunctionTable::new();
  basic::append_basic_functions(&mut table);
  arithmetic::append_arithmetic_functions(&mut table);
  table
}
