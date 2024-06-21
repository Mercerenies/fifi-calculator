
//! Functions that represent graphical data in the language.

use crate::expr::function::Function;
use crate::expr::function::builder::FunctionBuilder;
use crate::expr::function::table::FunctionTable;

pub fn append_graphics_functions(table: &mut FunctionTable) {
  table.insert(graphics_function());
  table.insert(plot_function());
}

/// The two-dimensional `graphics` directive. We don't actually define
/// any rules or simplifications for this function (similar to
/// datatypes like `vector`), but we do want the system to be aware of
/// this function.
pub fn graphics_function() -> Function {
  FunctionBuilder::new("graphics")
    .build()
}

pub fn plot_function() -> Function {
  FunctionBuilder::new("plot")
    .build()
}
