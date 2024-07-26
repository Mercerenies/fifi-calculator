
use crate::state::ApplicationState;
use crate::expr::Expr;
use crate::expr::number::Number;
use crate::expr::simplifier::{Simplifier, SimplifierContext};
use crate::expr::simplifier::identity::IdentitySimplifier;
use crate::expr::simplifier::error::SimplifierError;
use crate::errorlist::ErrorList;
use crate::units::parsing::{UnitParser, NullaryUnitParser};
use crate::mode::calculation::CalculationMode;
use super::options::CommandOptions;

pub trait Command {
  /// Runs the command. If a fatal error prevents the command from
  /// executing at all, then an `Err` should be returned. If the
  /// command executes, then an `Ok` should be returned, possibly
  /// reporting zero or more non-fatal errors using the
  /// [`CommandOutput`] object.
  fn run_command(
    &self,
    state: &mut ApplicationState,
    args: Vec<String>,
    ctx: &CommandContext,
  ) -> anyhow::Result<CommandOutput>;
}

pub struct CommandContext<'a, 'b> {
  pub opts: CommandOptions,
  pub simplifier: Box<dyn Simplifier + 'a>,
  pub units_parser: &'b dyn UnitParser<Number>,
  pub calculation_mode: CalculationMode,
}

/// The result of performing a command, including any non-fatal errors
/// that occurred.
#[derive(Debug, Clone)]
pub struct CommandOutput {
  errors: Vec<String>,
  force_scroll_down: bool,
}

impl<'a, 'b> CommandContext<'a, 'b> {
  pub fn simplify_expr(&self, expr: Expr, errors: &mut ErrorList<SimplifierError>) -> Expr {
    let mut simplifier_context = SimplifierContext {
      base_simplifier: self.simplifier.as_ref(),
      errors,
      calculation_mode: self.calculation_mode.clone(),
    };
    self.simplifier.simplify_expr(expr, &mut simplifier_context)
  }
}

impl CommandOutput {
  pub fn success() -> CommandOutput {
    CommandOutput {
      errors: vec![],
      force_scroll_down: true,
    }
  }

  pub fn from_errors<E, I>(errors: I) -> CommandOutput
  where I: IntoIterator<Item=E>,
        E: ToString {
    CommandOutput {
      errors: errors.into_iter().map(|e| e.to_string()).collect(),
      force_scroll_down: true,
    }
  }

  pub fn errors(&self) -> &[String] {
    &self.errors
  }

  /// Gets the error at the given index. Panics if out of bounds.
  pub fn get_error(&self, index: usize) -> &str {
    &self.errors[index]
  }

  /// Sets whether or not the UI for the stack should be forcibly
  /// scrolled down to the bottom when this command is done executing.
  /// The default is `true`.
  pub fn set_force_scroll_down(mut self, force_scroll_down: bool) -> Self {
    self.force_scroll_down = force_scroll_down;
    self
  }

  pub fn force_scroll_down(&self) -> bool {
    self.force_scroll_down
  }
}

/// An appropriate default context, with no special command options
/// and a simplifier that does nothing. Note carefully that this does
/// *not* provide a sensible simplifier and instead simply uses a
/// nullary one that returns its argument unmodified.
impl Default for CommandContext<'static, 'static> {
  fn default() -> Self {
    CommandContext {
      opts: CommandOptions::default(),
      simplifier: Box::new(IdentitySimplifier),
      units_parser: &NullaryUnitParser,
      calculation_mode: CalculationMode::default(),
    }
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn test_command_output_success() {
    let output = CommandOutput::success();
    assert!(output.errors.is_empty());
  }

  #[test]
  fn test_command_output_errors() {
    let output = CommandOutput::from_errors(vec!["X", "Y", "Z"]);
    assert_eq!(output.errors, vec!["X", "Y", "Z"]);
  }
}
