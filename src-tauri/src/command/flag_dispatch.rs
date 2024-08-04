
//! Commands which dispatch to other subcommands based on the values
//! of the Hyperbolic and Inverse modifier flags.
//!
//! The values of the modifier flags (and any part of
//! [`CommandOptions`]) are passed on unmodified to the subcommand.

use super::options::CommandOptions;
use super::subcommand::Subcommand;
use super::base::{Command, CommandContext, CommandOutput};
use crate::state::ApplicationState;

pub struct FlagDispatchCommand {
  no_flags: Box<dyn Command + Send + Sync>,
  hyper_flag: Option<Box<dyn Command + Send + Sync>>,
  inv_flag: Option<Box<dyn Command + Send + Sync>>,
  inv_hyper_flag: Option<Box<dyn Command + Send + Sync>>,
}

pub struct FlagDispatchArgs<C1, C2, C3, C4> {
  pub no_flags: C1,
  pub hyper_flag: C2,
  pub inv_flag: C3,
  pub inv_hyper_flag: C4,
}

impl FlagDispatchCommand {
  fn get_dispatch_command(&self, opts: &CommandOptions) -> &dyn Command {
    if opts.hyperbolic_modifier && opts.inverse_modifier {
      if let Some(inv_hyper) = &self.inv_hyper_flag {
        return inv_hyper.as_ref();
      }
    }
    if opts.hyperbolic_modifier {
      if let Some(hyper) = &self.hyper_flag {
        return hyper.as_ref();
      }
    }
    if opts.inverse_modifier {
      if let Some(inv) = &self.inv_flag {
        return inv.as_ref();
      }
    }
    self.no_flags.as_ref()
  }
}

impl Command for FlagDispatchCommand {
  fn run_command(
    &self,
    state: &mut ApplicationState,
    args: Vec<String>,
    ctx: &CommandContext,
  ) -> anyhow::Result<CommandOutput> {
    let command = self.get_dispatch_command(&ctx.opts);
    command.run_command(state, args, ctx)
  }

  fn as_subcommand(&self, opts: &CommandOptions) -> Option<Subcommand> {
    let command = self.get_dispatch_command(opts);
    command.as_subcommand(opts)
  }
}

pub fn dispatch_on_flags_command<C1, C2, C3, C4>(dispatch: FlagDispatchArgs<C1, C2, C3, C4>) -> FlagDispatchCommand
where C1: Command + Send + Sync + 'static,
      C2: Command + Send + Sync + 'static,
      C3: Command + Send + Sync + 'static,
      C4: Command + Send + Sync + 'static {
  FlagDispatchCommand {
    no_flags: Box::new(dispatch.no_flags),
    hyper_flag: Some(Box::new(dispatch.hyper_flag)),
    inv_flag: Some(Box::new(dispatch.inv_flag)),
    inv_hyper_flag: Some(Box::new(dispatch.inv_hyper_flag)),
  }
}

pub fn dispatch_on_hyper_command<C1, C2>(without_hyper: C1, with_hyper: C2) -> FlagDispatchCommand
where C1: Command + Send + Sync + 'static,
      C2: Command + Send + Sync + 'static {
  FlagDispatchCommand {
    no_flags: Box::new(without_hyper),
    hyper_flag: Some(Box::new(with_hyper)),
    inv_flag: None,
    inv_hyper_flag: None,
  }
}

pub fn dispatch_on_inverse_command<C1, C2>(without_inverse: C1, with_inverse: C2) -> FlagDispatchCommand
where C1: Command + Send + Sync + 'static,
      C2: Command + Send + Sync + 'static {
  FlagDispatchCommand {
    no_flags: Box::new(without_inverse),
    hyper_flag: None,
    inv_flag: Some(Box::new(with_inverse)),
    inv_hyper_flag: None,
  }
}

#[cfg(test)]
mod tests {
  use super::*;
  use crate::stack::test_utils::stack_of;
  use crate::command::test_utils::act_on_stack;
  use crate::command::functional::PushConstantCommand;
  use crate::expr::Expr;

  #[test]
  fn test_dispatch_on_flags_command() {
    let command = dispatch_on_flags_command(FlagDispatchArgs {
      no_flags: PushConstantCommand::new(Expr::from("no")),
      hyper_flag: PushConstantCommand::new(Expr::from("hyper")),
      inv_flag: PushConstantCommand::new(Expr::from("inv")),
      inv_hyper_flag: PushConstantCommand::new(Expr::from("inv_hyper")),
    });
    let input_stack = Vec::<Expr>::new();

    let opts = CommandOptions::default();
    let output_stack = act_on_stack(&command, opts, input_stack.clone()).unwrap();
    assert_eq!(output_stack, stack_of(vec!["no"]));

    let opts = CommandOptions::default().with_hyperbolic_modifier();
    let output_stack = act_on_stack(&command, opts, input_stack.clone()).unwrap();
    assert_eq!(output_stack, stack_of(vec!["hyper"]));

    let opts = CommandOptions::default().with_inverse_modifier();
    let output_stack = act_on_stack(&command, opts, input_stack.clone()).unwrap();
    assert_eq!(output_stack, stack_of(vec!["inv"]));

    let opts = CommandOptions::default().with_hyperbolic_modifier().with_inverse_modifier();
    let output_stack = act_on_stack(&command, opts, input_stack).unwrap();
    assert_eq!(output_stack, stack_of(vec!["inv_hyper"]));
  }

  #[test]
  fn test_dispatch_on_hyper_command() {
    let command = dispatch_on_hyper_command(
      PushConstantCommand::new(Expr::from("no")),
      PushConstantCommand::new(Expr::from("hyper")),
    );
    let input_stack = Vec::<Expr>::new();

    let opts = CommandOptions::default();
    let output_stack = act_on_stack(&command, opts, input_stack.clone()).unwrap();
    assert_eq!(output_stack, stack_of(vec!["no"]));

    let opts = CommandOptions::default().with_hyperbolic_modifier();
    let output_stack = act_on_stack(&command, opts, input_stack.clone()).unwrap();
    assert_eq!(output_stack, stack_of(vec!["hyper"]));

    let opts = CommandOptions::default().with_inverse_modifier();
    let output_stack = act_on_stack(&command, opts, input_stack.clone()).unwrap();
    assert_eq!(output_stack, stack_of(vec!["no"]));

    let opts = CommandOptions::default().with_hyperbolic_modifier().with_inverse_modifier();
    let output_stack = act_on_stack(&command, opts, input_stack).unwrap();
    assert_eq!(output_stack, stack_of(vec!["hyper"]));
  }

  #[test]
  fn test_dispatch_on_inv_command() {
    let command = dispatch_on_inverse_command(
      PushConstantCommand::new(Expr::from("no")),
      PushConstantCommand::new(Expr::from("inv")),
    );
    let input_stack = Vec::<Expr>::new();

    let opts = CommandOptions::default();
    let output_stack = act_on_stack(&command, opts, input_stack.clone()).unwrap();
    assert_eq!(output_stack, stack_of(vec!["no"]));

    let opts = CommandOptions::default().with_hyperbolic_modifier();
    let output_stack = act_on_stack(&command, opts, input_stack.clone()).unwrap();
    assert_eq!(output_stack, stack_of(vec!["no"]));

    let opts = CommandOptions::default().with_inverse_modifier();
    let output_stack = act_on_stack(&command, opts, input_stack.clone()).unwrap();
    assert_eq!(output_stack, stack_of(vec!["inv"]));

    let opts = CommandOptions::default().with_hyperbolic_modifier().with_inverse_modifier();
    let output_stack = act_on_stack(&command, opts, input_stack).unwrap();
    assert_eq!(output_stack, stack_of(vec!["inv"]));
  }
}
