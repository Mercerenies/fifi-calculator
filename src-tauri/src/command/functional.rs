
//! Commands that push functions onto the stack, using zero or more
//! arguments from the existing stack.

use super::base::{Command, CommandContext, CommandOutput};
use crate::state::ApplicationState;
use crate::stack::shuffle;
use crate::error::Error;
use crate::expr::Expr;
use crate::errorlist::ErrorList;

#[derive(Clone, Debug)]
pub struct PushConstantCommand {
  expr: Expr,
}

#[derive(Clone, Debug)]
pub struct UnaryFunctionCommand {
  function_name: String,
}

// Note for the future: I'm keeping these separate for now (rather
// than having a unified FunctionCommand which has an `arity`
// argument) since binary functions will have special treatment of
// prefix arguments.
#[derive(Clone, Debug)]
pub struct BinaryFunctionCommand {
  function_name: String,
}

impl PushConstantCommand {
  pub fn new(expr: Expr) -> PushConstantCommand {
    PushConstantCommand { expr }
  }
}

impl UnaryFunctionCommand {
  pub fn new(function_name: impl Into<String>) -> UnaryFunctionCommand {
    UnaryFunctionCommand { function_name: function_name.into() }
  }
}

impl BinaryFunctionCommand {
  pub fn new(function_name: impl Into<String>) -> BinaryFunctionCommand {
    BinaryFunctionCommand { function_name: function_name.into() }
  }
}

impl Command for PushConstantCommand {
  fn run_command(&self, state: &mut ApplicationState, ctx: &CommandContext) -> Result<CommandOutput, Error> {
    let arg = ctx.opts.argument.unwrap_or(1).min(0);
    let mut errors = ErrorList::new();
    for _ in 0..arg {
      state.main_stack.push(ctx.simplifier.simplify_expr(self.expr.clone(), &mut errors));
    }
    Ok(CommandOutput::from_errors(errors))
  }
}

impl Command for UnaryFunctionCommand {
  fn run_command(&self, state: &mut ApplicationState, ctx: &CommandContext) -> Result<CommandOutput, Error> {
    // TODO Use arg
    let mut errors = ErrorList::new();
    let top = shuffle::pop_one(&mut state.main_stack)?;
    let unary_call = Expr::Call(self.function_name.clone(), vec![top]);
    state.main_stack.push(ctx.simplifier.simplify_expr(unary_call, &mut errors));
    Ok(CommandOutput::from_errors(errors))
  }
}

impl Command for BinaryFunctionCommand {
  fn run_command(&self, state: &mut ApplicationState, ctx: &CommandContext) -> Result<CommandOutput, Error> {
    // TODO Use arg
    let mut errors = ErrorList::new();
    let (a, b) = shuffle::pop_two(&mut state.main_stack)?;
    let binary_call = Expr::Call(self.function_name.clone(), vec![a, b]);
    state.main_stack.push(ctx.simplifier.simplify_expr(binary_call, &mut errors));
    Ok(CommandOutput::from_errors(errors))
  }
}
