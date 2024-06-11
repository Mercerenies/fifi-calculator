
//! Functions that represent concrete data in the language. These
//! functions will usually not simplify, but they might have some
//! useful properties we can exploit.

use crate::expr::function::Function;
use crate::expr::function::builder::FunctionBuilder;
use crate::expr::function::table::FunctionTable;

pub fn append_datatype_functions(table: &mut FunctionTable) {
  table.insert(vector_function());
}

pub fn vector_function() -> Function {
  // Note: This function doesn't currently have any properties, but we
  // still recognize it.
  FunctionBuilder::new("vector")
    .build()
  // TODO: Derivative should be pointwise :)
}
