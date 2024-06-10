
//! Commands for invoking the calculus subsystems.

use super::arguments::{UnaryArgumentSchema, validate_schema};
use super::base::{Command, CommandContext, CommandOutput};
use crate::errorlist::ErrorList;
use crate::expr::Expr;
use crate::expr::atom::Atom;
use crate::expr::prisms::StringToVar;
use crate::expr::var::Var;
use crate::state::ApplicationState;
use crate::stack::base::StackLike;
use crate::stack::keepable::KeepableStack;

/// This command takes a variable `v` as an argument. When executed,
/// pops one value `expr` off the stack and pushes `deriv(expr, v)`,
/// which will attempt to calculate the derivative of the expression
/// `expr` in terms of the variable `v`.
///
/// Respects the "keep" modifier.
#[derive(Debug, Default)]
pub struct DerivativeCommand { // TODO: Multiple derivatives in one go
  _priv: (),
}

impl DerivativeCommand {
  pub fn new() -> Self {
    Default::default()
  }

  fn argument_schema() -> UnaryArgumentSchema<StringToVar, Var> {
    UnaryArgumentSchema::new(
      "variable name".to_owned(),
      StringToVar::new(),
    )
  }
}

impl Command for DerivativeCommand {
  fn run_command(
    &self,
    state: &mut ApplicationState,
    args: Vec<String>,
    context: &CommandContext,
  ) -> anyhow::Result<CommandOutput> {
    let variable_name = validate_schema(&DerivativeCommand::argument_schema(), args)?;

    let mut errors = ErrorList::new();
    state.undo_stack_mut().push_cut();

    let mut stack = KeepableStack::new(state.main_stack_mut(), context.opts.keep_modifier);
    let expr = stack.pop()?;
    let expr = Expr::call("deriv", vec![expr, Expr::Atom(Atom::Var(variable_name))]);
    let expr = context.simplifier.simplify_expr(expr, &mut errors);
    stack.push(expr);

    Ok(CommandOutput::from_errors(errors))
  }
}
