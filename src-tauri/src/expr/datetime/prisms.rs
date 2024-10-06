
//! Prisms relating to datetime values.

use super::{DATETIME_FUNCTION_NAME, DateTime};
use super::structure::{DatetimeValues, DateValues};
use crate::expr::Expr;
use crate::expr::prisms::{self, narrow_args, MatcherSpec, MatchedExpr};
use crate::util::prism::{Prism, PrismExt, Conversion};

use time::{Date, OffsetDateTime};
use either::Either;

/// This prism succeeds on call expressions whose head is `"datetime"`
/// and which has three arguments, all of them integers in the
/// appropriate range for the [`DateValues`] struct.
#[derive(Debug, Clone, Default)]
pub struct ExprToDateValues {
  _priv: (),
}

/// This prism succeeds on call expressions whose head is `"datetime"`
/// and which has eight arguments, all of them integers in the
/// appropriate range for the [`DatetimeValues`] struct.
#[derive(Debug, Clone, Default)]
pub struct ExprToDatetimeValues {
  _priv: (),
}

/// A [`MatcherSpec`] for arbitrary datetimes, whose arity is
/// insignificant.
#[derive(Debug)]
struct DatetimeMatcherSpec;

/// Type representing an arbitrary call to the `"datetime"` function,
/// which may or may not be a valid [`DateValues`] or
/// [`DatetimeValues`] instance. This type is often used as a
/// catch-all in calculator functions, to report an appropriate error
/// to the user when they enter a valid-looking but invalid date, such
/// as the `datetime` object which would represent the (invalid) date
/// April 31st.
#[derive(Debug, Clone)]
pub struct ArbitraryDatetime {
  original_expr: MatchedExpr<DatetimeMatcherSpec>,
}

pub fn expr_to_date() -> impl Prism<Expr, Date> + Clone {
  ExprToDateValues::new().composed(Conversion::new())
}

pub fn expr_to_offset_datetime() -> impl Prism<Expr, OffsetDateTime> + Clone {
  ExprToDatetimeValues::new().composed(Conversion::new())
}

pub fn expr_to_datetime() -> impl Prism<Expr, DateTime> + Clone {
  fn down(value: Either<Date, OffsetDateTime>) -> DateTime {
    value.either_into()
  }
  fn up(value: DateTime) -> Either<Date, OffsetDateTime> {
    if value.has_time() {
      Either::Right(value.to_offset_date_time().into())
    } else {
      Either::Left(value.date().into())
    }
  }
  expr_to_date()
    .or(expr_to_offset_datetime())
    .rmap(down, up)
}

pub fn expr_to_arbitrary_datetime() -> impl Prism<Expr, ArbitraryDatetime> + Clone {
  DatetimeMatcherSpec::prism().rmap(
    |matched_expr| ArbitraryDatetime { original_expr: matched_expr },
    |arbitrary_datetime| arbitrary_datetime.original_expr,
  )
}

impl ExprToDateValues {
  pub fn new() -> ExprToDateValues {
    ExprToDateValues { _priv: () }
  }
}

impl ExprToDatetimeValues {
  pub fn new() -> ExprToDatetimeValues {
    ExprToDatetimeValues { _priv: () }
  }
}

impl ArbitraryDatetime {
  pub fn expr(&self) -> &Expr {
    self.original_expr.as_ref()
  }
}

impl MatcherSpec for DatetimeMatcherSpec {
  const FUNCTION_NAME: &'static str = DATETIME_FUNCTION_NAME;
  const MIN_ARITY: usize = 0;
  const MAX_ARITY: usize = usize::MAX;
}

impl Prism<Expr, DateValues> for ExprToDateValues {
  fn narrow_type(&self, expr: Expr) -> Result<DateValues, Expr> {
    let prisms = (&prisms::expr_to_i32(), &prisms::expr_to_u8(), &prisms::expr_to_u8());
    narrow_args(DATETIME_FUNCTION_NAME, prisms, expr)
      .map(|(year, month, day)| DateValues { year, month, day })
  }

  fn widen_type(&self, date: DateValues) -> Expr {
    Expr::call(DATETIME_FUNCTION_NAME, vec![
      Expr::from(date.year as i64),
      Expr::from(date.month as i64),
      Expr::from(date.day as i64),
    ])
  }
}

impl Prism<Expr, DatetimeValues> for ExprToDatetimeValues {
  fn narrow_type(&self, expr: Expr) -> Result<DatetimeValues, Expr> {
    let prisms = (&prisms::expr_to_i32(), &prisms::expr_to_u8(), &prisms::expr_to_u8(),
                  &prisms::expr_to_u8(), &prisms::expr_to_u8(), &prisms::expr_to_u8(),
                  &prisms::expr_to_u32(), &prisms::expr_to_i32());
    narrow_args(DATETIME_FUNCTION_NAME, prisms, expr)
      .map(|(year, month, day, hour, minute, second, micro, offset)| {
        DatetimeValues { year, month, day, hour, minute, second, micro, offset }
      })
  }

  fn widen_type(&self, date: DatetimeValues) -> Expr {
    Expr::call(DATETIME_FUNCTION_NAME, vec![
      Expr::from(date.year as i64),
      Expr::from(date.month as i64),
      Expr::from(date.day as i64),
      Expr::from(date.hour as i64),
      Expr::from(date.minute as i64),
      Expr::from(date.second as i64),
      Expr::from(date.micro as i64),
      Expr::from(date.offset as i64),
    ])
  }
}

impl From<ArbitraryDatetime> for Expr {
  fn from(date: ArbitraryDatetime) -> Expr {
    date.original_expr.into()
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn test_narrow_to_date_values() {
    let expr = Expr::call("datetime", vec![Expr::from(10), Expr::from(20), Expr::from(30)]);
    let values = ExprToDateValues::new().narrow_type(expr).unwrap();
    assert_eq!(values, DateValues { year: 10, month: 20, day: 30 });

    let expr = Expr::call("datetime", vec![Expr::from(10), Expr::from(20)]);
    assert!(ExprToDateValues::new().narrow_type(expr).is_err());

    let expr = Expr::call("datetime", vec![Expr::from("some_string"), Expr::from(20), Expr::from(30)]);
    assert!(ExprToDateValues::new().narrow_type(expr).is_err());

    let expr = Expr::call("xyz", vec![Expr::from(10), Expr::from(20), Expr::from(30)]);
    assert!(ExprToDateValues::new().narrow_type(expr).is_err());
  }

  #[test]
  fn test_narrow_to_datetime_values() {
    let expr = Expr::call("datetime", vec![Expr::from(10), Expr::from(20), Expr::from(30), Expr::from(40),
                                           Expr::from(50), Expr::from(60), Expr::from(70), Expr::from(80)]);
    let values = ExprToDatetimeValues::new().narrow_type(expr).unwrap();
    assert_eq!(values, DatetimeValues { year: 10, month: 20, day: 30, hour: 40,
                                        minute: 50, second: 60, micro: 70, offset: 80 });

    let expr = Expr::call("datetime", vec![Expr::from(10), Expr::from(20), Expr::from(30)]);
    assert!(ExprToDatetimeValues::new().narrow_type(expr).is_err());
  }

  #[test]
  fn test_widen_from_date_values() {
    let values = DateValues { year: 10, month: 20, day: 30 };
    assert_eq!(
      ExprToDateValues::new().widen_type(values),
      Expr::call("datetime", vec![Expr::from(10), Expr::from(20), Expr::from(30)]),
    );
  }

  #[test]
  fn test_widen_from_datetime_values() {
    let values = DatetimeValues { year: 10, month: 20, day: 30, hour: 40,
                                  minute: 50, second: 60, micro: 70, offset: 80 };
    assert_eq!(
      ExprToDatetimeValues::new().widen_type(values),
      Expr::call("datetime", vec![Expr::from(10), Expr::from(20), Expr::from(30), Expr::from(40),
                                  Expr::from(50), Expr::from(60), Expr::from(70), Expr::from(80)]),
    );
  }

  #[test]
  fn test_narrow_to_arbitrary_datetime() {
    let expr = Expr::call("datetime", vec![Expr::from(10), Expr::from(20), Expr::from(30)]);
    let time = expr_to_arbitrary_datetime().narrow_type(expr.clone()).unwrap();
    assert_eq!(Expr::from(time), expr);

    let expr = Expr::call("datetime", vec![Expr::from(10), Expr::from(20), Expr::from(30), Expr::from(40)]);
    let time = expr_to_arbitrary_datetime().narrow_type(expr.clone()).unwrap();
    assert_eq!(Expr::from(time), expr);

    let expr = Expr::call("foo", vec![Expr::from(10)]);
    assert!(expr_to_arbitrary_datetime().narrow_type(expr).is_err());

    let expr = Expr::from("abc");
    assert!(expr_to_arbitrary_datetime().narrow_type(expr).is_err());
  }

  #[test]
  fn test_widen_from_arbitrary_datetime() {
    let expr = Expr::call("datetime", vec![Expr::from(10)]);
    let time = expr_to_arbitrary_datetime().narrow_type(expr.clone()).unwrap();
    assert_eq!(expr_to_arbitrary_datetime().widen_type(time), expr);
  }
}
