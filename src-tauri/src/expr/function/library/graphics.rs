
//! Functions that represent graphical data in the language.

use crate::expr::Expr;
use crate::expr::number::Number;
use crate::expr::function::Function;
use crate::expr::function::builder::{self, FunctionBuilder};
use crate::expr::function::table::FunctionTable;
use crate::expr::simplifier::error::SimplifierError;
use crate::expr::algebra::{ExprFunction, ExprFunction2};
use crate::expr::prisms;
use crate::util::{into_singleton, into_ordered};
use crate::util::prism::{Identity, Prism};
use crate::graphics::dataset::ExprToXDataSet;
use crate::graphics::plot::PlotDirective;
use crate::graphics::contour_plot::ContourPlotDirective;
use crate::graphics::response::GraphicsDirective;

pub fn append_graphics_functions(table: &mut FunctionTable) {
  table.insert(graphics_function());
  table.insert(plot_function());
  table.insert(contour_plot_function());
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
      builder::arity_two().of_types(ExprToXDataSet::new(), Identity).and_then(|x, y, ctx| {
        let Some(free_var) = into_singleton(y.clone().free_vars()) else {
          ctx.errors.push(SimplifierError::custom_error("plot", "expected a formula in one free variable"));
          return Err((x, y));
        };
        let is_parametric_function = PlotDirective::is_parametric_function(&y);
        let func = ExprFunction::new(y, free_var, ctx.simplifier);
        let plot =
          if is_parametric_function {
            PlotDirective::from_parametric_function(&x.into(), &func)
          } else {
            PlotDirective::from_expr_function(&x.into(), &func)
          };
        Ok(GraphicsDirective::Plot(plot))
      })
    )
    .build()
}

pub fn contour_plot_function() -> Function {
  FunctionBuilder::new("contourplot")
    .add_graphics_case(
      // Explicit vector of Z values.
      builder::arity_three().of_types(ExprToXDataSet::new(), ExprToXDataSet::new(), vec_vec_number_prism())
        .and_then(|x, y, z, ctx| {
          match ContourPlotDirective::from_points(&x.clone().into(), &y.clone().into(), z.clone()) {
            Err(err) => {
              ctx.errors.push(SimplifierError::new("contourplot", err));
              Err((x, y, z))
            }
            Ok(plot) => {
              Ok(GraphicsDirective::ContourPlot(plot))
            }
          }
        })
    )
    .add_graphics_case(
      // Formula in Z position.
      builder::arity_three().of_types(ExprToXDataSet::new(), ExprToXDataSet::new(), Identity)
        .and_then(|x, y, z, ctx| {
          let free_vars = into_ordered(z.clone().free_vars());
          let contour_plot = match free_vars.len() {
            1 => {
              let [var] = free_vars.try_into().unwrap();
              let func = ExprFunction::new(z, var, ctx.simplifier);
              ContourPlotDirective::from_complex_function(&x.into(), &y.into(), &func)
            }
            2 => {
              let [var1, var2] = free_vars.try_into().unwrap();
              let func = ExprFunction2::new(z, var1, var2, ctx.simplifier);
              ContourPlotDirective::from_expr_function2(&x.into(), &y.into(), &func)
            }
            _ => {
              ctx.errors.push(
                SimplifierError::custom_error("contourplot", "expected a formula in one or two free variables"),
              );
              return Err((x, y, z));
            }
          };
          Ok(GraphicsDirective::ContourPlot(contour_plot))
        })
    )
    .build()
}

fn vec_vec_number_prism() -> impl Prism<Expr, Vec<Vec<Number>>> {
  prisms::expr_to_typed_vector(
    prisms::expr_to_typed_vector(prisms::ExprToNumber),
  )
}
