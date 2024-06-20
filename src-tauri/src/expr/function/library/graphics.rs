
//! Functions that represent graphical data in the language. Like the
//! `datatypes` module, this module doesn't define very many concrete
//! rules or simplifications for these functions and merely exists to
//! make our engine aware of their existence.

use crate::expr::function::Function;
use crate::expr::function::builder::FunctionBuilder;
use crate::expr::function::table::FunctionTable;

pub fn append_graphics_functions(table: &mut FunctionTable) {
  table.insert(graphics_function());
  table.insert(plot_function());
}

pub fn graphics_function() -> Function {
  FunctionBuilder::new("graphics")
    .build()
}

pub fn plot_function() -> Function {
  FunctionBuilder::new("plot")
    .build()
}
