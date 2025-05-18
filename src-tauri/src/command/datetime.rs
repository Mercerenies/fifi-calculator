
//! Commands for working with datetimes.

use super::functional::UnaryFunctionCommand;
use crate::expr::Expr;
use crate::expr::datetime::DateTime;

use time::macros::date;

/// The `"days_since_zero"` command is defined such that Jan 1, 0001,
/// returns 1. This is the date we subtract to get that result.
pub const ZERO_DATE: DateTime =
  DateTime::from_date(date!(0000-12-31));

/// [`UnaryFunctionCommand`] which subtracts the given (constant) date
/// from its argument.
pub fn days_since_command(target_date: DateTime) -> UnaryFunctionCommand {
  UnaryFunctionCommand::new(move |arg| {
    Expr::call("-", vec![arg, target_date.clone().into()])
  })
}
