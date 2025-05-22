
//! Commands for working with datetimes.

use super::base::{Command, CommandContext, CommandOutput};
use super::options::CommandOptions;
use super::functional::UnaryFunctionCommand;
use super::arguments::{NullaryArgumentSchema, UnaryArgumentSchema, validate_schema};
use super::subcommand::Subcommand;
use crate::expr::Expr;
use crate::expr::number::Number;
use crate::expr::datetime::DateTime;
use crate::expr::datetime::parser::timezone::{TimezonePrism, ParsedTimezone};
use crate::state::ApplicationState;
use crate::stack::base::StackLike;
use crate::stack::keepable::KeepableStack;
use crate::errorlist::ErrorList;

use time::{OffsetDateTime, UtcOffset};
use time::macros::date;

/// Command which pushes the current datetime onto the stack.
///
/// With no numerical argument, returns the time in the current system
/// local timezone. A numerical argument is treated as a number of
/// hours offset from UTC.
#[derive(Debug, Default, Clone)]
pub struct NowCommand;

/// Command which converts a single date on top of the stack from one
/// timezone into another.
///
/// This command expects a single argument, which shall be a valid
/// timezone string according to
/// [`Timezone::parse`](crate::expr::datetime::parser::timezone::Timezone::parse).
///
/// Respects the "keep" modifier but ignores the numerical argument.
#[derive(Debug, Default, Clone)]
pub struct ConvertTimezoneCommand;

/// Command which pops a single element off the stack and invokes its
/// function with that element as the sole argument. With an explicit
/// numeric argument, that argument is passed as a second argument to
/// the invoked function.
///
/// Respects the "keep" modifier.
pub struct DatetimeIndexCommand {
  function: Box<DatetimeIndexFunction>,
}

pub type DatetimeIndexFunction = dyn Fn(Expr, Option<i64>) -> Expr + Send + Sync;

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

/// [`DatetimeIndexCommand`] which invokes `incmonth`. `incmonth` is
/// called with two arguments: the first is the top element of the
/// stack, and the second is the numerical argument times
/// `multiplicand`. If no numerical argument is supplied, the
/// numerical argument is treated as `1` (before multiplying by
/// `multiplicand`).
pub fn incmonth_command(multiplicand: i64) -> DatetimeIndexCommand {
  DatetimeIndexCommand::new(move |datetime, delta| {
    let delta = delta.unwrap_or(1).saturating_mul(multiplicand);
    Expr::call("incmonth", vec![datetime, delta.into()])
  })
}

impl ConvertTimezoneCommand {
  pub fn new() -> Self {
    Self
  }

  pub fn argument_schema() -> UnaryArgumentSchema<TimezonePrism, ParsedTimezone> {
    UnaryArgumentSchema::new(
      "valid timezone expression".to_owned(),
      TimezonePrism,
    )
  }
}

impl DatetimeIndexCommand {
  /// Delegates the top stack element and the optional numeric
  /// argument to the given pure Rust function.
  pub fn new(function: impl Fn(Expr, Option<i64>) -> Expr + Send + Sync + 'static) -> Self {
    Self {
      function: Box::new(function),
    }
  }

  /// Delegates the top stack element and the optional numeric
  /// argument to the given calculator function. The function should
  /// be callable with one or two arguments. It will be invoked with a
  /// single argument if the user did not supply an explicit numeric
  /// argument, or two if they did.
  pub fn named(function_name: impl Into<String>) -> Self {
    let function_name = function_name.into();
    Self::new(move |top, arg| {
      let args = if let Some(arg) = arg {
        vec![top, arg.into()]
      } else {
        vec![top]
      };
      Expr::call(function_name.clone(), args)
    })
  }

  fn wrap_expr(&self, top: Expr, arg: Option<i64>) -> Expr {
    (self.function)(top, arg)
  }
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

impl Command for ConvertTimezoneCommand {
  fn run_command(
    &self,
    state: &mut ApplicationState,
    args: Vec<String>,
    ctx: &CommandContext,
  ) -> anyhow::Result<CommandOutput> {
    let calculation_mode = state.calculation_mode().clone();
    let mut errors = ErrorList::new();

    let target_timezone = validate_schema(&Self::argument_schema(), args)?.timezone;
    state.undo_stack_mut().push_cut();
    let mut stack = KeepableStack::new(state.main_stack_mut(), ctx.opts.keep_modifier);
    let top = stack.pop()?;
    let whole_seconds = Number::from(target_timezone.0);
    let result = Expr::call("tzconvert", vec![top, whole_seconds.into()]);
    let result = ctx.simplify_expr(result, calculation_mode, &mut errors);
    stack.push(result);
    Ok(CommandOutput::from_errors(errors))
  }

  fn as_subcommand(&self, _opts: &CommandOptions) -> Option<Subcommand> {
    None
  }
}

impl Command for DatetimeIndexCommand {
  fn run_command(
    &self,
    state: &mut ApplicationState,
    args: Vec<String>,
    ctx: &CommandContext,
  ) -> anyhow::Result<CommandOutput> {
    validate_schema(&NullaryArgumentSchema::new(), args)?;

    let calculation_mode = state.calculation_mode().clone();
    let mut errors = ErrorList::new();

    state.undo_stack_mut().push_cut();
    let mut stack = KeepableStack::new(state.main_stack_mut(), ctx.opts.keep_modifier);
    let top = stack.pop()?;
    let result = self.wrap_expr(top, ctx.opts.argument);
    let result = ctx.simplify_expr(result, calculation_mode, &mut errors);
    stack.push(result);
    Ok(CommandOutput::from_errors(errors))
  }

  fn as_subcommand(&self, _opts: &CommandOptions) -> Option<Subcommand> {
    None
  }
}
