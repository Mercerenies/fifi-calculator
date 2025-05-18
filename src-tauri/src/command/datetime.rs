
//! Commands for working with datetimes.

use super::base::{Command, CommandContext, CommandOutput};
use super::options::CommandOptions;
use super::functional::UnaryFunctionCommand;
use super::arguments::{NullaryArgumentSchema, validate_schema};
use crate::expr::Expr;
use crate::expr::datetime::DateTime;
use crate::state::ApplicationState;
use crate::stack::base::StackLike;
use super::subcommand::Subcommand;

use time::{OffsetDateTime, UtcOffset};
use time::macros::date;

/// Command which pushes the current datetime onto the stack.
///
/// With no numerical argument, returns the time in the current system
/// local timezone. A numerical argument is treated as a number of
/// hours offset from UTC.
#[derive(Debug, Default, Clone)]
pub struct NowCommand;

/// The `"days_since_zero"` command is defined such that Jan 1, 0001,
/// returns 1. This is the date we subtract to get that result.
pub const ZERO_DATE: DateTime =
  DateTime::from_date(date!(0000-12-31));

/// Day "zero" for the purposes of Julian day calculations. The Julian
/// day of a date is defined as the number of days since Jan 1, 4713
/// BC on the Julian calendar. Using the proleptic Julian calendar,
/// this is Nov 24, 4713 BC.
pub const ZERO_JULIAN_DAY: DateTime =
  DateTime::from_date(date!(-4713-11-24));

/// The Unix epoch.
pub const UNIX_EPOCH: DateTime =
  DateTime::from_date(date!(1970-01-01));

/// [`UnaryFunctionCommand`] which converts a datetime to a number of
/// days relative to some constant date, or vice versa.
pub fn days_since_command(target_date: DateTime) -> UnaryFunctionCommand {
  UnaryFunctionCommand::new(move |arg| {
    Expr::call("datetime_rel", vec![arg, target_date.clone().into()])
  })
}

/// [`UnaryFunctionCommand`] which converts a datetime to a number of
/// seconds relative to some constant date, or vice versa.
pub fn secs_since_command(target_date: DateTime) -> UnaryFunctionCommand {
  UnaryFunctionCommand::new(move |arg| {
    Expr::call("datetime_rel_seconds", vec![arg, target_date.clone().into()])
  })
}

impl Command for NowCommand {
  fn run_command(
    &self,
    state: &mut ApplicationState,
    args: Vec<String>,
    ctx: &CommandContext,
  ) -> anyhow::Result<CommandOutput> {
    validate_schema(&NullaryArgumentSchema::new(), args)?;
    let now = if let Some(utc_offset) = ctx.opts.argument {
      let Some(utc_offset) = i8::try_from(utc_offset).ok().and_then(|i| UtcOffset::from_hms(i, 0, 0).ok()) else {
        anyhow::bail!("UTC offset out of range: {}", utc_offset);
      };
      let datetime = OffsetDateTime::now_utc().to_offset(utc_offset);
      DateTime::from(datetime)
    } else {
      DateTime::now_local()
    };
    state.undo_stack_mut().push_cut();
    let mut stack = state.main_stack_mut();
    stack.push(now.into());
    Ok(CommandOutput::success())
  }

  fn as_subcommand(&self, _opts: &CommandOptions) -> Option<Subcommand> {
    None
  }
}
