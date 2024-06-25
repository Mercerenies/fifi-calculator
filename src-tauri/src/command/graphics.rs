
//! Commands for invoking the algebra subsystems.

use super::arguments::{NullaryArgumentSchema, validate_schema};
use super::base::{Command, CommandContext, CommandOutput};
use crate::errorlist::ErrorList;
use crate::expr::Expr;
use crate::state::ApplicationState;
use crate::stack::base::StackLike;
use crate::stack::keepable::KeepableStack;
use crate::graphics::GRAPHICS_NAME;

/// This command pops two values off the stack. The top is treated as
/// the Y values and the next value down is treated as the X values.
/// Produces a two-dimensional graphics value which plots the given X
/// and Y coordinates.
///
/// Respects the "keep" modifier.
#[derive(Debug, Default)]
pub struct PlotCommand {
  _priv: (),
}

impl PlotCommand {
  pub fn new() -> Self {
    Default::default()
  }

  fn argument_schema() -> NullaryArgumentSchema {
    NullaryArgumentSchema::new()
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

    // TODO: Numerical arg

    let mut errors = ErrorList::new();
    state.undo_stack_mut().push_cut();

    let mut stack = KeepableStack::new(state.main_stack_mut(), context.opts.keep_modifier);
    let [x_values, y_values] = stack.pop_several(2)?.try_into().unwrap();
    let expr = Expr::call(GRAPHICS_NAME, vec![
      Expr::call("plot", vec![x_values, y_values]),
    ]);
    let expr = context.simplify_expr(expr, &mut errors);
    stack.push(expr);

    Ok(CommandOutput::from_errors(errors))
  }
}
