
//! Evaluation rules for datetime-related functions.

use crate::expr::prisms::{expr_to_number, expr_to_datetime};
use crate::expr::datetime::duration::PrecisionDuration;
use crate::expr::function::Function;
use crate::expr::function::table::FunctionTable;
use crate::expr::function::builder::{self, FunctionBuilder};
use crate::expr::simplifier::error::SimplifierError;
use crate::expr::number::Number;
use crate::expr::number::prisms::number_to_i64;
use crate::expr::number::inexact::DivInexact;
use crate::util::prism::Prism;

use num::{BigInt, ToPrimitive, Zero};

pub const MICROSECONDS_PER_DAY: i64 = 86_400_000_000;
pub const MICROSECONDS_PER_SECOND: i64 = 1_000_000;
pub const SECONDS_PER_DAY: i64 = 86_400;

pub fn append_datetime_functions(table: &mut FunctionTable) {
  table.insert(datetime_rel());
  table.insert(datetime_rel_seconds());
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
