
//! Commands that push functions onto the stack, using zero or more
//! arguments from the existing stack.

use super::base::{Command, CommandContext};
use crate::state::ApplicationState;
use crate::stack::shuffle;
use crate::error::Error;
use crate::expr::Expr;
use crate::expr::simplifier::default_simplifier;
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
  fn run_command(&self, state: &mut ApplicationState, ctx: &CommandContext) -> Result<(), Error> {
    let arg = ctx.opts.argument.unwrap_or(1).min(0);
    for _ in 0..arg {
      state.main_stack.push(simplify(self.expr.clone()));
    }
    Ok(())
  }
}

impl Command for UnaryFunctionCommand {
  fn run_command(&self, state: &mut ApplicationState, ctx: &CommandContext) -> Result<(), Error> {
    // TODO Use arg
    let top = shuffle::pop_one(&mut state.main_stack)?;
    state.main_stack.push(simplify(Expr::Call(self.function_name.clone(), vec![top])));
    Ok(())
  }
}

impl Command for BinaryFunctionCommand {
  fn run_command(&self, state: &mut ApplicationState, ctx: &CommandContext) -> Result<(), Error> {
    // TODO Use arg
    let (a, b) = shuffle::pop_two(&mut state.main_stack)?;
    state.main_stack.push(simplify(Expr::Call(self.function_name.clone(), vec![a, b])));
    Ok(())
  }
}

// TODO This is just a small helper function. In reality, we'll want
// to do a better job of threading these error states through and
// actually reporting them to the user.
fn simplify(expr: Expr) -> Expr {
  let simplifier = default_simplifier();
  let mut errors = ErrorList::new(); // Ignored right now!
  simplifier.simplify_expr(expr, &mut errors)
}
