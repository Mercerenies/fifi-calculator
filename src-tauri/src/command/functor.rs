
//! Commands for working with functors in an abstract sense. See also
//! [`library::functor`](crate::expr::function::library::functor).

use super::base::{Command, CommandContext, CommandOutput};
use super::options::CommandOptions;
use super::arguments::{NullaryArgumentSchema, validate_schema};
use super::subcommand::Subcommand;
use crate::expr::Expr;
use crate::state::ApplicationState;
use crate::stack::base::StackLike;
use crate::stack::keepable::KeepableStack;
use crate::errorlist::ErrorList;

/// The inverse of `fcompile`. Pops one value off the stack and pushes
/// its head and arguments, as two stack elements.
///
/// Respects the "keep" modifier.
#[derive(Debug, Default, Clone)]
pub struct FunctorUncompileCommand; // TODO Numeric argument

impl Command for FunctorUncompileCommand {
  fn run_command(
    &self,
    state: &mut ApplicationState,
    args: Vec<String>,
    ctx: &CommandContext,
  ) -> anyhow::Result<CommandOutput> {
    validate_schema(&NullaryArgumentSchema::new(), args)?;

    let calculation_mode = state.calculation_mode().clone();
    let mut errors = ErrorList::new();

    state.undo_stack_mut().push_cut();
    let mut stack = KeepableStack::new(state.main_stack_mut(), ctx.opts.keep_modifier);
    let top = stack.pop()?;

    let head_call = Expr::call("fhead", vec![top.clone()]);
    let head_call = ctx.simplify_expr(head_call, calculation_mode.clone(), &mut errors);

    let args_call = Expr::call("fargs", vec![top]);
    let args_call = ctx.simplify_expr(args_call, calculation_mode, &mut errors);

    stack.push(head_call);
    stack.push(args_call);
    Ok(CommandOutput::from_errors(errors))
  }

  fn as_subcommand(&self, _opts: &CommandOptions) -> Option<Subcommand> {
    None
  }
}
