
//! Specialized commands for working with variables in particular.

use super::arguments::{UnaryArgumentSchema, BinaryArgumentSchema, validate_schema};
use super::base::{Command, CommandContext, CommandOutput};
use super::options::CommandOptions;
use super::subcommand::Subcommand;
use crate::errorlist::ErrorList;
use crate::expr::Expr;
use crate::expr::prisms::StringToVar;
use crate::expr::var::Var;
use crate::expr::var::constants::validate_non_reserved_var_name;
use crate::util::prism::Identity;
use crate::state::ApplicationState;
use crate::state::undo::UpdateVarChange;
use crate::stack::base::StackLike;
use crate::stack::keepable::KeepableStack;

/// This command takes two arguments: a variable and an arbitrary
/// string (which will be parsed as an expression). Replaces all
/// instances of the given variable with the target expression in the
/// top stack element.
///
/// If the stack is empty, this command fails. Respects the "keep"
/// modifier of the command options but does not use the numerical
/// (prefix) argument.
///
/// Otherwise, replaces the top stack element with a call to the
/// `substitute(...)` function, to replace the argument variable with
/// the argument expression in the stack element.
#[derive(Debug, Default)]
pub struct SubstituteVarCommand {
  _priv: (),
}

/// This command takes one argument: the variable name into which to
/// store the top stack value. Fails if the top of the stack is empty.
///
/// Respects the "keep" modifier. If the "keep" modifier is false, the
/// top stack element will be popped.
#[derive(Debug, Default)]
pub struct StoreVarCommand {
  _priv: (),
}

/// This command takes one argument: a variable name. The command
/// unbinds the variable with the given name. If the variable was not
/// bound, nothing happens.
#[derive(Debug, Default)]
pub struct UnbindVarCommand {
  _priv: (),
}

impl SubstituteVarCommand {
  pub fn new() -> SubstituteVarCommand {
    SubstituteVarCommand { _priv: () }
  }

  fn argument_schema() -> BinaryArgumentSchema<StringToVar, Var, Identity, String> {
    BinaryArgumentSchema::new(
      "variable name".to_owned(),
      StringToVar::new(),
      "any argument".to_owned(),
      Identity,
    )
  }
}

impl StoreVarCommand {
  pub fn new() -> StoreVarCommand {
    StoreVarCommand { _priv: () }
  }

  fn argument_schema() -> UnaryArgumentSchema<StringToVar, Var> {
    UnaryArgumentSchema::new(
      "variable name".to_owned(),
      StringToVar::new(),
    )
  }
}

impl UnbindVarCommand {
  pub fn new() -> UnbindVarCommand {
    UnbindVarCommand { _priv: () }
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
  ) -> anyhow::Result<CommandOutput> {
    let calculation_mode = state.calculation_mode().clone();
    let (variable_name, new_value) = validate_schema(&SubstituteVarCommand::argument_schema(), args)?;

    let mut errors = ErrorList::new();
    state.undo_stack_mut().push_cut();

    let new_value = {
      let language_mode = &state.display_settings().language_mode();
      language_mode.parse(&new_value)?
    };

    let mut stack = KeepableStack::new(state.main_stack_mut(), context.opts.keep_modifier);
    let expr = stack.pop()?;
    let expr = Expr::call("substitute", vec![expr, Expr::from(variable_name), new_value]);
    let expr = context.simplify_expr(expr, calculation_mode, &mut errors);
    stack.push(expr);

    Ok(CommandOutput::from_errors(errors))
  }

  fn as_subcommand(&self, _opts: &CommandOptions) -> Option<Subcommand> {
    None
  }
}

impl Command for StoreVarCommand {
  fn run_command(
    &self,
    state: &mut ApplicationState,
    args: Vec<String>,
    context: &CommandContext,
  ) -> anyhow::Result<CommandOutput> {
    let variable_name = validate_schema(&StoreVarCommand::argument_schema(), args)?;

    validate_non_reserved_var_name(&variable_name)?;
    state.undo_stack_mut().push_cut();

    let old_value = state.variable_table().get(&variable_name).cloned();

    let mut stack = KeepableStack::new(state.main_stack_mut(), context.opts.keep_modifier);
    let expr = stack.pop()?;
    state.variable_table_mut().insert(variable_name.clone(), expr.clone());
    state.undo_stack_mut().push_change(UpdateVarChange::new(variable_name, old_value, Some(expr)));

    Ok(CommandOutput::success())
  }

  fn as_subcommand(&self, _opts: &CommandOptions) -> Option<Subcommand> {
    None
  }
}

impl Command for UnbindVarCommand {
  fn run_command(
    &self,
    state: &mut ApplicationState,
    args: Vec<String>,
    _context: &CommandContext,
  ) -> anyhow::Result<CommandOutput> {
    let variable_name = validate_schema(&UnbindVarCommand::argument_schema(), args)?;

    validate_non_reserved_var_name(&variable_name)?;
    state.undo_stack_mut().push_cut();

    let old_value = state.variable_table().get(&variable_name).cloned();

    state.variable_table_mut().remove(&variable_name);
    state.undo_stack_mut().push_change(UpdateVarChange::new(variable_name, old_value, None));

    Ok(CommandOutput::success())
  }

  fn as_subcommand(&self, _opts: &CommandOptions) -> Option<Subcommand> {
    None
  }
}
