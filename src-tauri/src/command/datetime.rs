
//! Commands for working with datetimes.

use super::functional::UnaryFunctionCommand;
use crate::expr::Expr;
use crate::expr::datetime::DateTime;

use time::macros::date;

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
