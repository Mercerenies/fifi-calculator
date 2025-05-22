
//! Evaluation rules for datetime-related functions.

use crate::expr::Expr;
use crate::expr::prisms::{expr_to_number, expr_to_datetime, expr_to_i32};
use crate::expr::datetime::DateTime;
use crate::expr::datetime::duration::PrecisionDuration;
use crate::expr::function::Function;
use crate::expr::function::table::FunctionTable;
use crate::expr::function::builder::{self, FunctionBuilder};
use crate::expr::simplifier::error::SimplifierError;
use crate::expr::number::Number;
use crate::expr::number::prisms::number_to_i64;
use crate::expr::number::inexact::DivInexact;
use crate::util::prism::Prism;

use num::{BigInt, ToPrimitive, Zero, clamp};
use time::{UtcOffset, Date, Duration, Month};
use time::util::{days_in_year, days_in_month};

pub const MICROSECONDS_PER_DAY: i64 = 86_400_000_000;
pub const MICROSECONDS_PER_SECOND: i64 = 1_000_000;
pub const SECONDS_PER_DAY: i64 = 86_400;

pub fn append_datetime_functions(table: &mut FunctionTable) {
  table.insert(datetime_rel());
  table.insert(datetime_rel_seconds());
  table.insert(tzconvert());
  table.insert(new_month());
  table.insert(new_year());
  table.insert(new_week());
  table.insert(inc_month());
}

// TODO Technically this is differentiable in its first argument (but
// not its second; it's kind of like conj() if we had treated it as a
// two-arg function).
//
// TODO A lot of this is redundant with the other arithmetic ops,
// sadly. Can we clean that up?
pub fn datetime_rel() -> Function {
  FunctionBuilder::new("datetime_rel")
    .add_case(
      // If given two datetime objects, equivalent to subtraction.
      builder::arity_two().both_of_type(expr_to_datetime()).and_then(|arg, rel, ctx| {
        let diff = arg - rel;
        if diff.is_precise() {
          let us = Number::from(diff.duration().whole_microseconds());
          let days = if ctx.calculation_mode.has_fractional_flag() {
            us / Number::from(MICROSECONDS_PER_DAY)
          } else {
            us.div_inexact(&Number::from(MICROSECONDS_PER_DAY))
          };
          Ok(days.into())
        } else {
          let whole_days = Number::from(diff.duration().whole_days());
          Ok(whole_days.into())
        }
      })
    )
    .add_case(
      // If given a real number and a datetime, equivalent to
      // addition.
      builder::arity_two().of_types(expr_to_number(), expr_to_datetime()).and_then(|arg, rel, ctx| {
        let Some(duration) = number_to_duration(arg.clone()) else {
          ctx.errors.push(SimplifierError::datetime_arithmetic_out_of_bounds("datetime_rel"));
          return Err((arg, rel));
        };
        let Some(result) = rel.clone().checked_add(duration) else {
          ctx.errors.push(SimplifierError::datetime_arithmetic_out_of_bounds("datetime_rel"));
          return Err((arg, rel));
        };
        Ok(result.into())
      })
    )
    .build()
}

pub fn datetime_rel_seconds() -> Function {
  FunctionBuilder::new("datetime_rel_seconds")
    .add_case(
      // If given two datetime objects, subtract and return number of
      // seconds.
      builder::arity_two().both_of_type(expr_to_datetime()).and_then(|arg, rel, ctx| {
        let diff = arg - rel;
        let diff_seconds = Number::from(diff.duration().whole_seconds());
        let diff_microseconds = Number::from(diff.duration().subsec_microseconds());
        if diff_microseconds.is_zero() {
          Ok(diff_seconds.into())
        } else {
          let mut total_seconds = diff_seconds + diff_microseconds / Number::from(MICROSECONDS_PER_SECOND);
          if !ctx.calculation_mode.has_fractional_flag() {
            total_seconds = total_seconds.to_inexact();
          }
          Ok(total_seconds.into())
        }
      })
    )
    .add_case(
      // If given a real number and a datetime, return number of
      // seconds since that date.
      builder::arity_two().of_types(expr_to_number(), expr_to_datetime()).and_then(|arg, rel, ctx| {
        let Some(duration) = number_to_duration(arg.clone() / Number::from(SECONDS_PER_DAY)) else {
          ctx.errors.push(SimplifierError::datetime_arithmetic_out_of_bounds("datetime_rel"));
          return Err((arg, rel));
        };
        let Some(result) = rel.clone().checked_add(duration) else {
          ctx.errors.push(SimplifierError::datetime_arithmetic_out_of_bounds("datetime_rel"));
          return Err((arg, rel));
        };
        Ok(result.into())
      })
    )
    .build()
}

pub fn tzconvert() -> Function {
  FunctionBuilder::new("tzconvert")
    .add_case(
      // Datetime and new UTC offset in seconds
      builder::arity_two().of_types(expr_to_datetime(), expr_to_i32()).and_then(|arg, offset_sec, ctx| {
        let Ok(offset) = UtcOffset::from_whole_seconds(offset_sec) else {
          ctx.errors.push(SimplifierError::custom_error("tzconvert", "Invalid UTC offset"));
          return Err((arg, offset_sec));
        };
        let new_datetime = arg.to_offset_date_time().to_offset(offset);
        let new_datetime = DateTime::from(new_datetime);
        Ok(Expr::from(new_datetime))
      })
    )
    .build()
}

pub fn new_month() -> Function {
  fn day_of_month(datetime: DateTime, mut day_index: i32) -> DateTime {
    let datetime = datetime.without_time();
    let days_in_month = datetime.month().length(datetime.year()) as i32;

    if day_index <= 0 {
      // Count from the last day of the month
      day_index += days_in_month + 1;
    }
    let day_index = clamp(day_index, 1, days_in_month) as u8; // safety: clamp is between 1 and a u8.
    let result_date = Date::from_calendar_date(datetime.year(), datetime.month(), day_index)
      .expect("day_index is between 1 and days_in_month");
    result_date.into()
  }

  FunctionBuilder::new("newmonth")
    .add_case(
      // Single datetime argument returns first day of month.
      builder::arity_one().of_type(expr_to_datetime()).and_then(|arg, _| {
        Ok(Expr::from(day_of_month(arg, 1)))
      })
    )
    .add_case(
      // datetime + i32 = nth day of month.
      builder::arity_two().of_types(expr_to_datetime(), expr_to_i32()).and_then(|arg, idx, _| {
        Ok(Expr::from(day_of_month(arg, idx)))
      })
    )
    .build()
}

pub fn new_year() -> Function {
  const MIN_YEAR: i32 = Date::MIN.year();
  const MAX_YEAR: i32 = Date::MAX.year();

  // Precondition: `year` is in the range `MIN_YEAR..=MAX_YEAR`
  fn day_of_year(year: i32, mut day_index: i32) -> DateTime {
    assert!((MIN_YEAR..=MAX_YEAR).contains(&year));

    let days_in_year = i32::from(days_in_year(year));

    if day_index <= 0 {
      // Count from the last day of the year
      day_index += days_in_year + 1;
    }
    let day_index = clamp(day_index, 1, days_in_year) as u16; // safety: clamp is between 1 and a u16.
    let result_date = Date::from_ordinal_date(year, day_index)
      .expect("day_index is between 1 and days_in_year");
    result_date.into()
  }

  FunctionBuilder::new("newyear")
    .add_case(
      // Single datetime argument returns first day of year.
      builder::arity_one().of_type(expr_to_datetime()).and_then(|arg, _| {
        Ok(Expr::from(day_of_year(arg.year(), 1)))
      })
    )
    .add_case(
      // datetime + i32 = nth day of year.
      builder::arity_two().of_types(expr_to_datetime(), expr_to_i32()).and_then(|arg, idx, _| {
        Ok(Expr::from(day_of_year(arg.year(), idx)))
      })
    )
    .add_case(
      // Single integer argument returns first day of integer year.
      builder::arity_one().of_type(expr_to_i32()).and_then(|arg, ctx| {
        if !(MIN_YEAR..=MAX_YEAR).contains(&arg) {
          ctx.errors.push(SimplifierError::custom_error("newyear", "Year out of range"));
          return Err(arg);
        }
        Ok(Expr::from(day_of_year(arg, 1)))
      })
    )
    .add_case(
      // Two integer arguments returns nth day of integer year.
      builder::arity_two().both_of_type(expr_to_i32()).and_then(|arg, idx, ctx| {
        if !(MIN_YEAR..=MAX_YEAR).contains(&arg) {
          ctx.errors.push(SimplifierError::custom_error("newyear", "Year out of range"));
          return Err((arg, idx));
        }
        Ok(Expr::from(day_of_year(arg, idx)))
      })
    )
    .build()
}

pub fn new_week() -> Function {
  const DAYS_IN_WEEK: i32 = 7;

  fn day_of_week(datetime: DateTime, mut day_index: i32) -> DateTime {
    let datetime = datetime.without_time();

    if day_index <= 0 {
      // Count from the last day of the month
      day_index += DAYS_IN_WEEK + 1;
    }
    let day_index = clamp(day_index, 1, DAYS_IN_WEEK) as u8; // safety: clamp is between 1 and 7.
    let beginning_of_week = datetime - Duration::days((datetime.weekday().number_days_from_sunday()).into());
    let result_date = beginning_of_week + Duration::days((day_index - 1).into());
    result_date.into()
  }

  FunctionBuilder::new("newweek")
    .add_case(
      // Single datetime argument returns first day of week.
      builder::arity_one().of_type(expr_to_datetime()).and_then(|arg, _| {
        Ok(Expr::from(day_of_week(arg, 1)))
      })
    )
    .add_case(
      // datetime + i32 = nth day of week.
      builder::arity_two().of_types(expr_to_datetime(), expr_to_i32()).and_then(|arg, idx, _| {
        Ok(Expr::from(day_of_week(arg, idx)))
      })
    )
    .build()
}

pub fn inc_month() -> Function {
  const DATETIME_OUT_OF_BOUNDS: &'static str = "Datetime out of bounds";

  FunctionBuilder::new("incmonth")
    .add_case(
      // Datetime plus integer number of months
      builder::arity_two().of_types(expr_to_datetime(), expr_to_i32()).and_then(|datetime, delta_months, ctx| {
        let arg_date = datetime.without_time();

        let year = arg_date.year();
        let month = i32::from(u8::from(arg_date.month()));
        let Some(month) = month.checked_add(delta_months) else {
          ctx.errors.push(SimplifierError::custom_error("incmonth", DATETIME_OUT_OF_BOUNDS));
          return Err((datetime, delta_months));
        };

        let (year, month) = simplify_year_month(year, month);
        let month = Month::try_from(month as u8).expect("month is between 1 and 12");

        let day = clamp(arg_date.day(), 1, days_in_month(month, year));
        let Ok(result_date) = Date::from_calendar_date(year, month, day) else {
          ctx.errors.push(SimplifierError::custom_error("incmonth", DATETIME_OUT_OF_BOUNDS));
          return Err((datetime, delta_months));
        };
        let result_datetime = datetime.replace_date(result_date);
        Ok(Expr::from(result_datetime))
      })
    )
    .build()
}

pub(super) fn number_to_duration(n: Number) -> Option<PrecisionDuration> {
  match number_to_i64().narrow_type(n) {
    Err(n) => {
      // Precise (down to microseconds) duration
      let microseconds = (Number::from(MICROSECONDS_PER_DAY) * n).floor();
      let microseconds = BigInt::try_from(microseconds).expect("floor() always produces an integer");
      microseconds.to_i64().map(PrecisionDuration::microseconds)
    }
    Ok(i) => {
      // Imprecise (day-level) duration
      Some(PrecisionDuration::days(i))
    }
  }
}

/// If `month` is outside the range `1..=12`, adjusts the year by an
/// appropriate number of years to make the month fit within that
/// range.
fn simplify_year_month(year: i32, month: i32) -> (i32, i32) {
  if month < 1 {
    (year - ((-month + 12) / 12), (month % 12 + 11) % 12 + 1)
  } else if month > 12 {
    (year + (month - 1) / 12, (month - 1) % 12 + 1)
  } else {
    (year, month)
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn test_simplify_year_month() {
    assert_eq!(simplify_year_month(2000, 1), (2000, 1));
    assert_eq!(simplify_year_month(2000, 4), (2000, 4));
    assert_eq!(simplify_year_month(2000, 12), (2000, 12));

    assert_eq!(simplify_year_month(2000, 0), (1999, 12));
    assert_eq!(simplify_year_month(2000, -1), (1999, 11));
    assert_eq!(simplify_year_month(2000, -10), (1999, 2));
    assert_eq!(simplify_year_month(2000, -11), (1999, 1));
    assert_eq!(simplify_year_month(2000, -12), (1998, 12));

    assert_eq!(simplify_year_month(2000, 13), (2001, 1));
    assert_eq!(simplify_year_month(2000, 20), (2001, 8));
    assert_eq!(simplify_year_month(2000, 23), (2001, 11));
    assert_eq!(simplify_year_month(2000, 24), (2001, 12));
    assert_eq!(simplify_year_month(2000, 25), (2002, 1));
    assert_eq!(simplify_year_month(2000, 27), (2002, 3));
    assert_eq!(simplify_year_month(2000, 87), (2007, 3));
  }
}
