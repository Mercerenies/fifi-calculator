
//! Defines a command struct which, when executed, parses and pushes
//! user input to the stack.

use super::base::{Command, CommandContext, CommandOutput};
use super::arguments::{UnaryArgumentSchema, validate_schema};
use super::options::CommandOptions;
use super::subcommand::Subcommand;
use crate::util::prism::Identity;
use crate::util::cow_dyn::CowDyn;
use crate::expr::Expr;
use crate::expr::number::Number;
use crate::expr::simplifier::Simplifier;
use crate::expr::simplifier::dollar_sign::DollarSignRefSimplifier;
use crate::state::ApplicationState;
use crate::stack::base::StackLike;
use crate::mode::display::language::LanguageMode;
use crate::errorlist::ErrorList;

use std::str::FromStr;

/// A command which parses and pushes user input to the stack. Parsing
/// is done via a custom function supplied by the constructor.
///
/// On parse failure, the stack is not modified.
pub struct PushInputCommand<F, SF> {
  body: F,
  simplifier_function: SF,
}

impl<F> PushInputCommand<F, for<'s> fn(&'s (dyn Simplifier + 's), &'s ApplicationState) -> CowDyn<'s, dyn Simplifier + 's>>
where F: Fn(String, &dyn LanguageMode) -> anyhow::Result<Expr> {
  pub fn new(body: F) -> Self {
    Self {
      body,
      simplifier_function: |simpl, _| CowDyn::Borrowed(simpl),
    }
  }
}

impl<F, SF> PushInputCommand<F, SF>
where F: Fn(String, &dyn LanguageMode) -> anyhow::Result<Expr>,
      SF: for<'s> Fn(&'s (dyn Simplifier + 's), &'s ApplicationState) -> CowDyn<'s, dyn Simplifier + 's> {
  pub fn with_simplifier<SF1>(self, simplifier_function: SF1) -> PushInputCommand<F, SF1>
  where SF1: for<'s> Fn(&'s (dyn Simplifier + 's), &'s ApplicationState) -> CowDyn<'s, dyn Simplifier + 's> {
    PushInputCommand {
      body: self.body,
      simplifier_function,
    }
  }

  pub fn try_parse(&self, arg: String, language_mode: &dyn LanguageMode) -> anyhow::Result<Expr> {
    (self.body)(arg, language_mode)
  }
}

impl<F, SF> Command for PushInputCommand<F, SF>
where F: Fn(String, &dyn LanguageMode) -> anyhow::Result<Expr>,
      SF: for<'s> Fn(&'s (dyn Simplifier + 's), &'s ApplicationState) -> CowDyn<'s, dyn Simplifier + 's> {
  fn run_command(
    &self,
    state: &mut ApplicationState,
    args: Vec<String>,
    context: &CommandContext,
  ) -> anyhow::Result<CommandOutput> {
    let calculation_mode = state.calculation_mode().clone();
    let arg = validate_schema(&argument_schema(), args)?;
    let mut errors = ErrorList::new();

    state.undo_stack_mut().push_cut();
    let expr = self.try_parse(arg, state.display_settings().language_mode().as_ref())?;
    let expr = context.simplify_expr_using(expr, calculation_mode, &mut errors, |base_simplifier| {
      (self.simplifier_function)(base_simplifier, state)
    });
    state.main_stack_mut().push(expr);
    Ok(CommandOutput::from_errors(errors))
  }

  fn as_subcommand(&self, _opts: &CommandOptions) -> Option<Subcommand> {
    None
  }
}

fn argument_schema() -> UnaryArgumentSchema<Identity, String> {
  UnaryArgumentSchema::any()
}

/// A `PushInputCommand` which parses a literal real number and pushes
/// it onto the stack.
pub fn push_number_command() -> impl Command {
  PushInputCommand::new(|arg, _| {
    let number = Number::from_str(&arg)?;
    Ok(Expr::from(number))
  })
}

/// A `PushInputCommand` which uses the current language mode to parse
/// a general expression.
pub fn push_expr_command() -> impl Command {
  PushInputCommand::new(|arg, language_mode| language_mode.parse(&arg)).with_simplifier(|base_simplifier, state| {
    CowDyn::Owned(Box::new(DollarSignRefSimplifier::prepended(state.main_stack(), base_simplifier)))
  })
}

/// A `PushInputCommand` which pushes a literal string onto the stack.
/// Always succeeds.
pub fn push_string_command() -> impl Command {
  PushInputCommand::new(|arg, _| Ok(Expr::from(arg)))
}

#[cfg(test)]
mod tests {
  use super::*;
  use crate::command::test_utils::act_on_stack;
  use crate::stack::Stack;
  use crate::stack::test_utils::stack_of;
  use crate::state::test_utils::state_for_stack;

  #[test]
  fn test_push_number_command() {
    let input_stack = vec![10, 20, 30];
    let output_stack = act_on_stack(
      &push_number_command(),
      vec!["400"],
      input_stack,
    ).unwrap();
    assert_eq!(output_stack, stack_of(vec![10, 20, 30, 400]));
  }

  #[test]
  fn test_push_number_command_failure() {
    let mut state = state_for_stack(vec![10, 20, 30]);
    let context = CommandContext::default();
    let err = push_number_command().run_command(&mut state, vec![String::from("a")], &context).unwrap_err();
    assert_eq!(state.into_main_stack(), stack_of(vec![10, 20, 30])); // Stack should be unchanged.
    assert_eq!(err.to_string(), "Failed to parse number");
  }

  #[test]
  fn test_push_expr_command_with_simple_number() {
    let input_stack = vec![10, 20, 30];
    let output_stack = act_on_stack(
      &push_expr_command(),
      vec!["400"],
      input_stack,
    ).unwrap();
    assert_eq!(output_stack, stack_of(vec![10, 20, 30, 400]));
  }

  #[test]
  fn test_push_expr_command_with_stack_references() {
    let input_stack = vec![10, 20, 30];
    let output_stack = act_on_stack(
      &push_expr_command(),
      vec!["$1 - $2"],
      input_stack,
    ).unwrap();
    assert_eq!(output_stack, stack_of(vec![
      Expr::from(10),
      Expr::from(20),
      Expr::from(30),
      Expr::call("-", vec![Expr::from(30), Expr::from(20)]),
    ]));
  }

  #[test]
  fn test_push_expr_command_with_complex_expr() {
    let input_stack = vec![10, 20, 30];
    let output_stack = act_on_stack(
      &push_expr_command(),
      vec!["x + y"],
      input_stack,
    ).unwrap();
    assert_eq!(
      output_stack,
      Stack::from(vec![
        Expr::from(10),
        Expr::from(20),
        Expr::from(30),
        Expr::call("+", vec![Expr::var("x").unwrap(), Expr::var("y").unwrap()]),
      ]),
    )
  }

  #[test]
  fn test_push_expr_command_failure() {
    let mut state = state_for_stack(vec![10, 20, 30]);
    let context = CommandContext::default();
    let err = push_expr_command().run_command(&mut state, vec![String::from("x +")], &context).unwrap_err();
    assert_eq!(state.into_main_stack(), stack_of(vec![10, 20, 30])); // Stack should be unchanged.
    assert_eq!(err.to_string(), "Operator parsing error: Failed to parse operator chain: + at 2-3");
  }

  #[test]
  fn test_push_string_command() {
    let input_stack = vec![10, 20, 30];
    let output_stack = act_on_stack(
      &push_string_command(),
      vec!["hello"],
      input_stack,
    ).unwrap();
    assert_eq!(output_stack, stack_of(vec![
      Expr::from(10),
      Expr::from(20),
      Expr::from(30),
      Expr::from("hello"),
    ]));
  }
}
