
use crate::state::ApplicationState;
use crate::expr::Expr;
use crate::expr::number::Number;
use crate::expr::simplifier::{Simplifier, SimplifierContext};
use crate::expr::simplifier::identity::IdentitySimplifier;
use crate::expr::simplifier::error::SimplifierError;
use crate::errorlist::ErrorList;
use crate::units::parsing::{UnitParser, NullaryUnitParser};
use crate::mode::calculation::CalculationMode;
use crate::util::cow_dyn::CowDyn;
use super::options::CommandOptions;
use super::subcommand::Subcommand;
use super::dispatch::CommandDispatchTable;

use once_cell::sync::Lazy;

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

  /// Converts the command into a [`Subcommand`], if possible.
  ///
  /// Many commands can be run as subcommands of another command,
  /// usually one that operates over a collection, such as a fold or a
  /// map operation. For such commands, this method produces the
  /// subcommand that performs the same basic functionality as the
  /// current command. On commands for which this notion doesn't make
  /// sense, this method should return `None`, which will produce a
  /// user-facing error if the user tries to use the command as a
  /// subcommand.
  ///
  /// Subcommands should, generally speaking, respect the hyperbolic
  /// and inverse modifiers of the options field. That is, if a
  /// command does something different when invoked with the
  /// hyperbolic modifier, then the produced subcommand should be
  /// different if given the hyperbolic modifier (and, respectively,
  /// the inverse modifier).
  ///
  /// Variants based on the numerical prefix argument are left to the
  /// discretion of the command. Some commands use the presence or
  /// absence of the numerical argument as a flag, similar to the
  /// hyperbolic or inverse modifiers. In that case, the subcommand
  /// should also be dispatched based on the same flag. For commands
  /// which use the numerical argument as a sort of arity or
  /// indication of the number of arguments to pop off the stack, the
  /// corresponding subcommand will generally ignore the numerical
  /// argument.
  fn as_subcommand(&self, opts: &CommandOptions) -> Option<Subcommand>;
}

pub struct CommandContext<'a, 'b, 'c> {
  pub opts: CommandOptions,
  pub simplifier: Box<dyn Simplifier + 'a>,
  pub units_parser: &'b dyn UnitParser<Number>,
  pub dispatch_table: &'c CommandDispatchTable,
}

/// The result of performing a command, including any non-fatal errors
/// that occurred.
#[derive(Debug, Clone)]
pub struct CommandOutput {
  errors: Vec<String>,
  force_scroll_down: bool,
}

impl<'a, 'b, 'c> CommandContext<'a, 'b, 'c> {
  pub fn simplify_expr(
    &self,
    expr: Expr,
    calculation_mode: CalculationMode,
    errors: &mut ErrorList<SimplifierError>,
  ) -> Expr {
    self.simplify_expr_using(expr, calculation_mode, errors, CowDyn::Borrowed)
  }

  pub fn simplify_expr_using<'s>(
    &'s self,
    expr: Expr,
    calculation_mode: CalculationMode,
    errors: &mut ErrorList<SimplifierError>,
    custom_simplifier_fn: impl FnOnce(&'s (dyn Simplifier + 'a)) -> CowDyn<'s, dyn Simplifier + 'a>,
  ) -> Expr {
    let simplifier = custom_simplifier_fn(self.simplifier.as_ref());
    let mut simplifier_context = SimplifierContext {
      base_simplifier: simplifier.as_ref(),
      errors,
      calculation_mode,
    };
    simplifier.simplify_expr(expr, &mut simplifier_context)
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
impl Default for CommandContext<'static, 'static, 'static> {
  fn default() -> Self {
    static EMPTY_DISPATCH_TABLE: Lazy<CommandDispatchTable> = Lazy::new(CommandDispatchTable::default);

    CommandContext {
      opts: CommandOptions::default(),
      simplifier: Box::new(IdentitySimplifier),
      units_parser: &NullaryUnitParser,
      dispatch_table: &EMPTY_DISPATCH_TABLE,
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
