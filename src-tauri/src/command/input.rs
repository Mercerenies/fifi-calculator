
//! Defines a command struct which, when executed, parses and pushes
//! user input to the stack.

use super::base::{Command, CommandContext, CommandOutput};
use super::arguments::{UnaryArgumentSchema, validate_schema};
use crate::util::prism::Identity;
use crate::expr::Expr;
use crate::error::Error;
use crate::state::ApplicationState;
use crate::stack::base::StackLike;
use crate::display::language::LanguageMode;
use crate::errorlist::ErrorList;

/// A command which parses and pushes user input to the stack. Parsing
/// is done via a custom function supplied by the constructor.
///
/// On parse failure, the stack is not modified.
pub struct PushInputCommand<F> {
  body: F,
}

impl<F> PushInputCommand<F>
where F: Fn(String, &dyn LanguageMode) -> Result<Expr, Error> {
  pub fn new(body: F) -> Self {
    Self { body }
  }

  pub fn try_parse(&self, arg: String, language_mode: &dyn LanguageMode) -> Result<Expr, Error> {
    (self.body)(arg, language_mode)
  }
}

impl<F> Command for PushInputCommand<F>
where F: Fn(String, &dyn LanguageMode) -> Result<Expr, Error> {
  fn run_command(
    &self,
    state: &mut ApplicationState,
    args: Vec<String>,
    context: &CommandContext,
  ) -> Result<CommandOutput, Error> {
    let schema = argument_schema();
    let arg = validate_schema(schema, args)?;
    let mut errors = ErrorList::new();

    state.undo_stack_mut().push_cut();
    let expr = self.try_parse(arg, state.display_settings().language_mode.as_ref())?;
    let expr = context.simplifier.simplify_expr(expr, &mut errors);
    state.main_stack_mut().push(expr);
    Ok(CommandOutput::from_errors(errors))
  }
}

fn argument_schema() -> UnaryArgumentSchema<Identity, String> {
  UnaryArgumentSchema::any()
}
