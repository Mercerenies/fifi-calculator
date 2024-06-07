
use crate::expr::function::Function;
use crate::expr::function::builder::{self, FunctionBuilder};
use crate::expr::function::table::FunctionTable;

pub fn append_basic_functions(table: &mut FunctionTable) {
  table.insert(identity_function());
}

pub fn identity_function() -> Function {
  FunctionBuilder::new("identity")
    .add_case(
      builder::arity_one().and_then(|arg, _| Ok(arg))
    )
    .build()
}
