
//! Functions that represent graphical data in the language.

use crate::expr::function::Function;
use crate::expr::function::builder::{self, FunctionBuilder};
use crate::expr::function::table::FunctionTable;
use crate::expr::simplifier::error::SimplifierError;
use crate::expr::algebra::ExprFunction;
use crate::expr::prisms;
use crate::util::into_singleton;
use crate::util::prism::Identity;
use crate::graphics::dataset::ExprToXDataSet;
use crate::graphics::plot::PlotDirective;
use crate::graphics::response::GraphicsDirective;

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
    .add_graphics_case(
      // X dataset with explicit vector of Y coordinates.
      builder::arity_two().of_types(ExprToXDataSet::new(), prisms::expr_to_typed_vector(prisms::ExprToNumber))
        .and_then(|x, y, ctx| {
          match PlotDirective::from_points(&x.clone().into(), &y) {
            Err(err) => {
              ctx.errors.push(SimplifierError::new("plot", err));
              Err((x, y))
            }
            Ok(plot) => {
              Ok(GraphicsDirective::Plot(plot))
            }
          }
        })
    )
    .add_graphics_case(
      // X dataset with formula in Y position.
      builder::arity_two().of_types(ExprToXDataSet::new(), Identity::new()).and_then(|x, y, ctx| {
        let Some(free_var) = into_singleton(y.clone().free_vars()) else {
          ctx.errors.push(SimplifierError::custom_error("plot", "expected a formula in one free variable"));
          return Err((x, y));
        };
        let func = ExprFunction::new(y, free_var, ctx.simplifier);
        let plot = PlotDirective::from_expr_function(&x.into(), &func);
        Ok(GraphicsDirective::Plot(plot))
      })
    )
    .build()
}
