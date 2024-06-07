
use crate::state::ApplicationState;
use crate::expr::simplifier::Simplifier;
use crate::expr::simplifier::identity::IdentitySimplifier;
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

pub struct CommandContext<'a> {
  pub opts: CommandOptions,
  pub simplifier: Box<dyn Simplifier + 'a>,
}

/// The result of performing a command, including any non-fatal errors
/// that occurred.
#[derive(Debug, Clone)]
pub struct CommandOutput {
  pub errors: Vec<String>,
}

impl CommandOutput {
  pub fn success() -> CommandOutput {
    CommandOutput {
      errors: vec![],
    }
  }

  pub fn from_errors<E, I>(errors: I) -> CommandOutput
  where I: IntoIterator<Item=E>,
        E: ToString {
    CommandOutput {
      errors: errors.into_iter().map(|e| e.to_string()).collect(),
    }
  }
}

/// An appropriate default context, with no special command options
/// and a simplifier that does nothing. Note carefully that this does
/// *not* provide a sensible simplifier and instead simply uses a
/// nullary one that returns its argument unmodified.
impl Default for CommandContext<'static> {
  fn default() -> Self {
    CommandContext {
      opts: CommandOptions::default(),
      simplifier: Box::new(IdentitySimplifier),
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
