
//! Specialized commands for working with variables in particular.

use super::arguments::{UnaryArgumentSchema, BinaryArgumentSchema, validate_schema};
use super::base::{Command, CommandContext, CommandOutput};
use crate::errorlist::ErrorList;
use crate::expr::prisms::StringToVar;
use crate::expr::var::Var;
use crate::util::prism::Identity;
use crate::state::ApplicationState;
use crate::state::undo::UpdateVarChange;
use crate::stack::base::StackLike;
use crate::stack::keepable::KeepableStack;
use crate::error::Error;

use std::marker::PhantomData;

/// This command takes two arguments: a variable and an arbitrary
/// string (which will be parsed as an expression). Replaces all
/// instances of the given variable with the target expression in the
/// top stack element.
///
/// If the stack is empty, this command fails. It is NOT an error for
/// the variable to be absent from the target stack expression. In
/// that case, the stack value is unchanged. This command is also
/// inherently single-pass, so a substitution can be self-referencing.
/// That is, it's meaningful to replace `x` with `x + 1` using this
/// function, since the `x` on the right-hand side will not get
/// recursively substituted.
///
/// Respects the "keep" modifier of the command options but does not
/// use the numerical (prefix) argument.
#[derive(Debug)]
pub struct SubstituteVarCommand {
  _priv: PhantomData<()>,
}

/// This command takes one argument: the variable name into which to
/// store the top stack value. Fails if the top of the stack is empty.
///
/// Respects the "keep" modifier. If the "keep" modifier is false, the
/// top stack element will be popped.
#[derive(Debug)]
pub struct StoreVarCommand {
  _priv: PhantomData<()>,
}

impl SubstituteVarCommand {
  pub fn new() -> SubstituteVarCommand {
    SubstituteVarCommand { _priv: PhantomData }
  }

  fn argument_schema() -> BinaryArgumentSchema<StringToVar, Var, Identity, String> {
    BinaryArgumentSchema::new(
      "variable name".to_owned(),
      StringToVar::new(),
      "any argument".to_owned(),
      Identity::new(),
    )
  }
}

impl StoreVarCommand {
  pub fn new() -> StoreVarCommand {
    StoreVarCommand { _priv: PhantomData }
  }

  fn argument_schema() -> UnaryArgumentSchema<StringToVar, Var> {
    UnaryArgumentSchema::new(
      "variable name".to_owned(),
      StringToVar::new(),
    )
  }
}

impl Command for SubstituteVarCommand {
  fn run_command(
    &self,
    state: &mut ApplicationState,
    args: Vec<String>,
    context: &CommandContext,
  ) -> Result<CommandOutput, Error> {
    let (variable_name, new_value) = validate_schema(&SubstituteVarCommand::argument_schema(), args)?;

    let mut errors = ErrorList::new();
    state.undo_stack_mut().push_cut();

    let language_mode = &state.display_settings().language_mode;
    let new_value = language_mode.parse(&new_value)?;

    let mut stack = KeepableStack::new(state.main_stack_mut(), context.opts.keep_modifier);
    let expr = stack.pop()?;
    let expr = expr.substitute_var(variable_name, new_value);
    let expr = context.simplifier.simplify_expr(expr, &mut errors);
    stack.push(expr);

    Ok(CommandOutput::from_errors(errors))
  }
}

impl Command for StoreVarCommand {
  fn run_command(
    &self,
    state: &mut ApplicationState,
    args: Vec<String>,
    context: &CommandContext,
  ) -> Result<CommandOutput, Error> {
    let variable_name = validate_schema(&StoreVarCommand::argument_schema(), args)?;

    state.undo_stack_mut().push_cut();

    let old_value = state.variable_table().get(&variable_name).cloned();

    let mut stack = KeepableStack::new(state.main_stack_mut(), context.opts.keep_modifier);
    let expr = stack.pop()?;
    state.variable_table_mut().insert(variable_name.clone(), expr.clone());
    state.undo_stack_mut().push_change(UpdateVarChange::new(variable_name, old_value, Some(expr)));

    Ok(CommandOutput::success())
  }
}
