
//! Commands for invoking the algebra subsystems.

use super::arguments::{UnaryArgumentSchema, validate_schema};
use super::base::{Command, CommandContext, CommandOutput};
use super::subcommand::Subcommand;
use super::options::CommandOptions;
use crate::errorlist::ErrorList;
use crate::expr::Expr;
use crate::expr::atom::Atom;
use crate::expr::prisms::StringToVar;
use crate::expr::var::Var;
use crate::state::ApplicationState;
use crate::stack::base::StackLike;
use crate::stack::keepable::KeepableStack;

/// This command takes a variable `v` as an argument. When executed,
/// pops two values `expr` and `guess` off the stack and pushes
/// `find_root(expr, v, guess)`.
///
/// Respects the "keep" modifier.
#[derive(Debug, Default)]
pub struct FindRootCommand {
  _priv: (),
}

impl FindRootCommand {
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

impl Command for FindRootCommand {
  fn run_command(
    &self,
    state: &mut ApplicationState,
    args: Vec<String>,
    context: &CommandContext,
  ) -> anyhow::Result<CommandOutput> {
    let calculation_mode = state.calculation_mode().clone();
    let variable_name = validate_schema(&FindRootCommand::argument_schema(), args)?;

    // TODO: Should the numerical argument do anything for this command?

    let mut errors = ErrorList::new();
    state.undo_stack_mut().push_cut();

    let mut stack = KeepableStack::new(state.main_stack_mut(), context.opts.keep_modifier);
    let [expr, guess] = stack.pop_several(2)?.try_into().unwrap();
    let expr = Expr::call("find_root", vec![expr, Expr::Atom(Atom::Var(variable_name)), guess]);
    let expr = context.simplify_expr(expr, calculation_mode, &mut errors);
    stack.push(expr);

    Ok(CommandOutput::from_errors(errors))
  }

  fn as_subcommand(&self, _opts: &CommandOptions) -> Option<Subcommand> {
    None
  }
}
