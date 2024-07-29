
//! Commands which dispatch to other subcommands based on the values
//! of the Hyperbolic and Inverse modifier flags.
//!
//! The values of the modifier flags (and any part of
//! [`CommandOptions`]) are passed on unmodified to the subcommand.

use super::general::GeneralCommand;
use super::base::Command;

pub struct FlagDispatchArgs<C1, C2, C3, C4> {
  pub no_flags: C1,
  pub hyper_flag: C2,
  pub inv_flag: C3,
  pub inv_hyper_flag: C4,
}

pub fn dispatch_on_flags_command<C1, C2, C3, C4>(dispatch: FlagDispatchArgs<C1, C2, C3, C4>) -> impl Command
where C1: Command,
      C2: Command,
      C3: Command,
      C4: Command {
  GeneralCommand::new(move |state, args, ctx| {
    match (ctx.opts.hyperbolic_modifier, ctx.opts.inverse_modifier) {
      (false, false) => dispatch.no_flags.run_command(state, args, ctx),
      (false, true) => dispatch.inv_flag.run_command(state, args, ctx),
      (true, false) => dispatch.hyper_flag.run_command(state, args, ctx),
      (true, true) => dispatch.inv_hyper_flag.run_command(state, args, ctx),
    }
  })
}

pub fn dispatch_on_hyper_command<C1, C2>(without_hyper: C1, with_hyper: C2) -> impl Command
where C1: Command,
      C2: Command {
  GeneralCommand::new(move |state, args, ctx| {
    if ctx.opts.hyperbolic_modifier {
      with_hyper.run_command(state, args, ctx)
    } else {
      without_hyper.run_command(state, args, ctx)
    }
  })
}

pub fn dispatch_on_inverse_command<C1, C2>(without_inverse: C1, with_inverse: C2) -> impl Command
where C1: Command,
      C2: Command {
  GeneralCommand::new(move |state, args, ctx| {
    if ctx.opts.inverse_modifier {
      with_inverse.run_command(state, args, ctx)
    } else {
      without_inverse.run_command(state, args, ctx)
    }
  })
}
