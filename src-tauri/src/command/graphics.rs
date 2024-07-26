
//! Commands for invoking the algebra subsystems.

use super::arguments::{NullaryArgumentSchema, validate_schema};
use super::base::{Command, CommandContext, CommandOutput};
use crate::errorlist::ErrorList;
use crate::expr::Expr;
use crate::expr::vector::ExprToVector;
use crate::expr::prisms::expr_to_typed_array;
use crate::state::ApplicationState;
use crate::stack::base::StackLike;
use crate::stack::keepable::KeepableStack;
use crate::graphics::GRAPHICS_NAME;
use crate::util::prism::{Prism, Identity, OnVec};

use std::cmp::Ordering;

/// This command pops two values off the stack. The top is treated as
/// the Y values and the next value down is treated as the X values.
/// Produces a two-dimensional graphics value which plots the given X
/// and Y coordinates.
///
/// If given a positive numerical argument N, then N + 1 values are
/// popped, the top of which is treated as a shared X coordinate
/// dataset. A total of N graphs will be produced in this case. A lack
/// of numerical argument is treated as a numerical argument of 1.
///
/// If the numerical argument is zero, then two values are popped. The
/// first is the X dataset, and the second must be a vector. Each
/// element of the latter vector is treated as a separate Y dataset
/// for plotting purposes.
///
/// If the numerical argument is some negative value -N, then N values
/// are popped from the stack. Each value must be a vector of two
/// elements, which are treated as the X- and Y- coordinates of a
/// plot.
///
/// Respects the "keep" modifier.
#[derive(Debug, Default)]
pub struct PlotCommand {
  _priv: (),
}

/// This command pops three values off the stack: An X interval, a Y
/// interval, and an output dataset. Produces a two-dimensional
/// graphics value which acts as a contour plot.
///
/// If given an explicit numerical argument, then the output dataset
/// shall produce complex numbers. In this case, the output dataset
/// must be a formula (not a vector of vectors), and two contour plots
/// are pushed onto the stack. These two plots represent the real and
/// imaginary components of the output dataset, respectively.
///
/// Respects the "keep" modifier.
#[derive(Debug, Default)]
pub struct ContourPlotCommand {
  _priv: (),
}

impl PlotCommand {
  pub fn new() -> Self {
    Default::default()
  }

  fn argument_schema() -> NullaryArgumentSchema {
    NullaryArgumentSchema::new()
  }

  fn basic_plot(x_values: Expr, y_values: Expr) -> Expr {
    Expr::call("plot", vec![x_values, y_values])
  }
}

impl ContourPlotCommand {
  pub fn new() -> Self {
    Default::default()
  }

  fn argument_schema() -> NullaryArgumentSchema {
    NullaryArgumentSchema::new()
  }

  fn contour_plot(x_values: Expr, y_values: Expr, output_dataset: Expr) -> Expr {
    Expr::call("contourplot", vec![x_values, y_values, output_dataset])
  }
}

impl Command for PlotCommand {
  fn run_command(
    &self,
    state: &mut ApplicationState,
    args: Vec<String>,
    context: &CommandContext,
  ) -> anyhow::Result<CommandOutput> {
    validate_schema(&PlotCommand::argument_schema(), args)?;

    let calculation_mode = state.calculation_mode().clone();

    let arg = context.opts.argument.unwrap_or(1);
    let mut errors = ErrorList::new();
    state.undo_stack_mut().push_cut();
    let mut stack = KeepableStack::new(state.main_stack_mut(), context.opts.keep_modifier);

    match arg.cmp(&0) {
      Ordering::Greater => {
        // Pop N y-values and one x-value.
        let (x_values, y_values_vec) = {
          let mut all_values = stack.pop_several((arg + 1) as usize)?;
          let x_values = all_values.remove(0);
          (x_values, all_values)
        };
        let expr = Expr::call(
          GRAPHICS_NAME,
          y_values_vec.into_iter().map(|y_values| PlotCommand::basic_plot(x_values.clone(), y_values)).collect(),
        );
        let expr = context.simplify_expr(expr, calculation_mode, &mut errors);
        stack.push(expr);
      }
      Ordering::Less => {
        // Pop N 2-vectors of x- and y- values.
        let all_values: Vec<Expr> = stack.pop_several((- arg) as usize)?;
        let xy_values: Vec<[Expr; 2]> =
          match OnVec::new(expr_to_typed_array(Identity)).narrow_type(all_values) {
            Err(all_values) => {
              // Failure, restore the stack and report an error.
              if !context.opts.keep_modifier {
                stack.push_several(all_values);
              }
              anyhow::bail!("Expecting 2-vectors of X and Y values");
            }
            Ok(xy_values) => xy_values,
          };
        let expr = Expr::call(
          GRAPHICS_NAME,
          xy_values.into_iter().map(|[x_values, y_values]| PlotCommand::basic_plot(x_values, y_values)).collect(),
        );
        let expr = context.simplify_expr(expr, calculation_mode, &mut errors);
        stack.push(expr);
      }
      Ordering::Equal => {
        // Pop two values, y-values must be a vector.
        let [x_values, y_values] = stack.pop_several(2)?.try_into().unwrap();
        match ExprToVector.narrow_type(y_values) {
          Err(y_values) => {
            // Failure, restore the stack and report an error.
            if !context.opts.keep_modifier {
              stack.push_several([x_values, y_values]);
            }
            anyhow::bail!("Expecting vector of Y values");
          }
          Ok(y_values_vec) => {
            let expr = Expr::call(
              GRAPHICS_NAME,
              y_values_vec.into_iter().map(|y_values| PlotCommand::basic_plot(x_values.clone(), y_values)).collect(),
            );
            let expr = context.simplify_expr(expr, calculation_mode, &mut errors);
            stack.push(expr);
          }
        }
      }
    }

    Ok(CommandOutput::from_errors(errors))
  }
}

impl Command for ContourPlotCommand {
  fn run_command(
    &self,
    state: &mut ApplicationState,
    args: Vec<String>,
    context: &CommandContext,
  ) -> anyhow::Result<CommandOutput> {
    validate_schema(&ContourPlotCommand::argument_schema(), args)?;

    let calculation_mode = state.calculation_mode().clone();

    let should_produce_vector = context.opts.argument.is_some();
    let mut errors = ErrorList::new();
    state.undo_stack_mut().push_cut();
    let mut stack = KeepableStack::new(state.main_stack_mut(), context.opts.keep_modifier);

    if should_produce_vector {
      // Vector of two contour plots
      let [x_values, y_values, data_values] = stack.pop_several(3)?.try_into().unwrap();
      let real_data_values = Expr::call("re", vec![data_values.clone()]);
      let imag_data_values = Expr::call("im", vec![data_values]);
      // Real part
      let expr = Expr::call(
        GRAPHICS_NAME,
        vec![
          ContourPlotCommand::contour_plot(x_values.clone(), y_values.clone(), real_data_values),
        ],
      );
      let expr = context.simplify_expr(expr, calculation_mode.clone(), &mut errors);
      stack.push(expr);
      // Imag part
      let expr = Expr::call(
        GRAPHICS_NAME,
        vec![
          ContourPlotCommand::contour_plot(x_values, y_values, imag_data_values),
        ],
      );
      let expr = context.simplify_expr(expr, calculation_mode, &mut errors);
      stack.push(expr);
    } else {
      // Single contour plot
      let [x_values, y_values, data_values] = stack.pop_several(3)?.try_into().unwrap();
      let expr = Expr::call(
        GRAPHICS_NAME,
        vec![ContourPlotCommand::contour_plot(x_values, y_values, data_values)],
      );
      let expr = context.simplify_expr(expr, calculation_mode, &mut errors);
      stack.push(expr);
    }

    Ok(CommandOutput::from_errors(errors))
  }
}

#[cfg(test)]
mod tests {
  use super::*;
  use crate::command::test_utils::act_on_stack;
  use crate::command::options::CommandOptions;
  use crate::stack::test_utils::stack_of;

  #[test]
  fn test_basic_plot_command() {
    let opts = CommandOptions::default();
    let input_stack = vec![10, 20];
    let output_stack = act_on_stack(&PlotCommand::new(), opts, input_stack).unwrap();
    assert_eq!(output_stack, stack_of(vec![
      Expr::call("graphics", vec![Expr::call("plot", vec![Expr::from(10), Expr::from(20)])]),
    ]));
  }

  #[test]
  fn test_basic_plot_command_with_keep_modifier() {
    let opts = CommandOptions::default().with_keep_modifier();
    let input_stack = vec![10, 20];
    let output_stack = act_on_stack(&PlotCommand::new(), opts, input_stack).unwrap();
    assert_eq!(output_stack, stack_of(vec![
      Expr::from(10),
      Expr::from(20),
      Expr::call("graphics", vec![Expr::call("plot", vec![Expr::from(10), Expr::from(20)])]),
    ]));
  }

  #[test]
  fn test_plot_command_with_positive_prefix_arg() {
    let opts = CommandOptions::numerical(2);
    let input_stack = vec![10, 20, 30, 40];
    let output_stack = act_on_stack(&PlotCommand::new(), opts, input_stack).unwrap();
    assert_eq!(output_stack, stack_of(vec![
      Expr::from(10),
      Expr::call("graphics", vec![
        Expr::call("plot", vec![Expr::from(20), Expr::from(30)]),
        Expr::call("plot", vec![Expr::from(20), Expr::from(40)]),
      ]),
    ]));
  }

  #[test]
  fn test_plot_command_with_positive_prefix_arg_and_keep_arg() {
    let opts = CommandOptions::numerical(2).with_keep_modifier();
    let input_stack = vec![10, 20, 30, 40];
    let output_stack = act_on_stack(&PlotCommand::new(), opts, input_stack).unwrap();
    assert_eq!(output_stack, stack_of(vec![
      Expr::from(10),
      Expr::from(20),
      Expr::from(30),
      Expr::from(40),
      Expr::call("graphics", vec![
        Expr::call("plot", vec![Expr::from(20), Expr::from(30)]),
        Expr::call("plot", vec![Expr::from(20), Expr::from(40)]),
      ]),
    ]));
  }

  #[test]
  fn test_plot_command_with_zero_prefix_arg() {
    let opts = CommandOptions::numerical(0);
    let input_stack = vec![
      Expr::from(10),
      Expr::call("vector", vec![Expr::from(20), Expr::from(30)]),
    ];
    let output_stack = act_on_stack(&PlotCommand::new(), opts, input_stack).unwrap();
    assert_eq!(output_stack, stack_of(vec![
      Expr::call("graphics", vec![
        Expr::call("plot", vec![Expr::from(10), Expr::from(20)]),
        Expr::call("plot", vec![Expr::from(10), Expr::from(30)]),
      ]),
    ]));
  }

  #[test]
  fn test_plot_command_with_zero_prefix_arg_and_keep_modifier() {
    let opts = CommandOptions::numerical(0).with_keep_modifier();
    let input_stack = vec![
      Expr::from(10),
      Expr::call("vector", vec![Expr::from(20), Expr::from(30)]),
    ];
    let output_stack = act_on_stack(&PlotCommand::new(), opts, input_stack).unwrap();
    assert_eq!(output_stack, stack_of(vec![
      Expr::from(10),
      Expr::call("vector", vec![Expr::from(20), Expr::from(30)]),
      Expr::call("graphics", vec![
        Expr::call("plot", vec![Expr::from(10), Expr::from(20)]),
        Expr::call("plot", vec![Expr::from(10), Expr::from(30)]),
      ]),
    ]));
  }

  #[test]
  fn test_plot_command_with_zero_prefix_arg_and_invalid_y_arg() {
    let opts = CommandOptions::numerical(0);
    let input_stack = vec![
      Expr::from(10),
      Expr::from(20),
    ];
    let err = act_on_stack(&PlotCommand::new(), opts, input_stack).unwrap_err();
    assert_eq!(err.to_string(), "Expecting vector of Y values");
  }

  #[test]
  fn test_plot_command_with_negative_prefix_arg() {
    let opts = CommandOptions::numerical(-3);
    let input_stack = vec![
      Expr::from(10),
      Expr::call("vector", vec![Expr::from(20), Expr::from(30)]),
      Expr::call("vector", vec![Expr::from(40), Expr::from(50)]),
      Expr::call("vector", vec![Expr::from(60), Expr::from(70)]),
    ];
    let output_stack = act_on_stack(&PlotCommand::new(), opts, input_stack).unwrap();
    assert_eq!(output_stack, stack_of(vec![
      Expr::from(10),
      Expr::call("graphics", vec![
        Expr::call("plot", vec![Expr::from(20), Expr::from(30)]),
        Expr::call("plot", vec![Expr::from(40), Expr::from(50)]),
        Expr::call("plot", vec![Expr::from(60), Expr::from(70)]),
      ]),
    ]));
  }

  #[test]
  fn test_plot_command_with_negative_prefix_arg_and_keep_modifier() {
    let opts = CommandOptions::numerical(-3).with_keep_modifier();
    let input_stack = vec![
      Expr::from(10),
      Expr::call("vector", vec![Expr::from(20), Expr::from(30)]),
      Expr::call("vector", vec![Expr::from(40), Expr::from(50)]),
      Expr::call("vector", vec![Expr::from(60), Expr::from(70)]),
    ];
    let output_stack = act_on_stack(&PlotCommand::new(), opts, input_stack).unwrap();
    assert_eq!(output_stack, stack_of(vec![
      Expr::from(10),
      Expr::call("vector", vec![Expr::from(20), Expr::from(30)]),
      Expr::call("vector", vec![Expr::from(40), Expr::from(50)]),
      Expr::call("vector", vec![Expr::from(60), Expr::from(70)]),
      Expr::call("graphics", vec![
        Expr::call("plot", vec![Expr::from(20), Expr::from(30)]),
        Expr::call("plot", vec![Expr::from(40), Expr::from(50)]),
        Expr::call("plot", vec![Expr::from(60), Expr::from(70)]),
      ]),
    ]));
  }

  #[test]
  fn test_plot_command_with_zero_prefix_arg_and_non_vector_stack_value() {
    let opts = CommandOptions::numerical(-4);
    let input_stack = vec![
      Expr::from(10),
      Expr::call("vector", vec![Expr::from(20), Expr::from(30)]),
      Expr::call("vector", vec![Expr::from(40), Expr::from(50)]),
      Expr::call("vector", vec![Expr::from(60), Expr::from(70)]),
    ];
    let err = act_on_stack(&PlotCommand::new(), opts, input_stack).unwrap_err();
    assert_eq!(err.to_string(), "Expecting 2-vectors of X and Y values");
  }

  #[test]
  fn test_contour_plot_command() {
    let opts = CommandOptions::default();
    let input_stack = vec![10, 20, 30, 40];
    let output_stack = act_on_stack(&ContourPlotCommand::new(), opts, input_stack).unwrap();
    assert_eq!(output_stack, stack_of(vec![
      Expr::from(10),
      Expr::call("graphics", vec![Expr::call("contourplot", vec![Expr::from(20), Expr::from(30), Expr::from(40)])]),
    ]));
  }

  #[test]
  fn test_contour_plot_command_with_numerical_arg() {
    let opts = CommandOptions::numerical(10);
    let input_stack = vec![10, 20, 30, 40];
    let output_stack = act_on_stack(&ContourPlotCommand::new(), opts, input_stack).unwrap();
    assert_eq!(output_stack, stack_of(vec![
      Expr::from(10),
      Expr::call("graphics", vec![
        Expr::call("contourplot", vec![Expr::from(20), Expr::from(30), Expr::call("re", vec![Expr::from(40)])]),
      ]),
      Expr::call("graphics", vec![
        Expr::call("contourplot", vec![Expr::from(20), Expr::from(30), Expr::call("im", vec![Expr::from(40)])]),
      ]),
    ]));
  }

  #[test]
  fn test_contour_plot_command_with_keep_arg() {
    let opts = CommandOptions::default().with_keep_modifier();
    let input_stack = vec![10, 20, 30, 40];
    let output_stack = act_on_stack(&ContourPlotCommand::new(), opts, input_stack).unwrap();
    assert_eq!(output_stack, stack_of(vec![
      Expr::from(10),
      Expr::from(20),
      Expr::from(30),
      Expr::from(40),
      Expr::call("graphics", vec![Expr::call("contourplot", vec![Expr::from(20), Expr::from(30), Expr::from(40)])]),
    ]));
  }

  #[test]
  fn test_contour_plot_command_with_numerical_arg_and_keep_arg() {
    let opts = CommandOptions::numerical(10).with_keep_modifier();
    let input_stack = vec![10, 20, 30, 40];
    let output_stack = act_on_stack(&ContourPlotCommand::new(), opts, input_stack).unwrap();
    assert_eq!(output_stack, stack_of(vec![
      Expr::from(10),
      Expr::from(20),
      Expr::from(30),
      Expr::from(40),
      Expr::call("graphics", vec![
        Expr::call("contourplot", vec![Expr::from(20), Expr::from(30), Expr::call("re", vec![Expr::from(40)])]),
      ]),
      Expr::call("graphics", vec![
        Expr::call("contourplot", vec![Expr::from(20), Expr::from(30), Expr::call("im", vec![Expr::from(40)])]),
      ]),
    ]));
  }
}
