
//! Functions that represent concrete data in the language. These
//! functions will usually not simplify, but they might have some
//! useful properties we can exploit.

use crate::expr::Expr;
use crate::expr::function::Function;
use crate::expr::function::builder::FunctionBuilder;
use crate::expr::function::table::FunctionTable;

pub fn append_datatype_functions(table: &mut FunctionTable) {
  table.insert(vector_function());
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
