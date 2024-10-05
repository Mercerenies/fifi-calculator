
//! Prisms relating to datetime values.

use super::DATETIME_FUNCTION_NAME;
use super::structure::{DatetimeValues, DateValues};
use crate::expr::Expr;
use crate::expr::prisms::{self, narrow_args};
use crate::util::prism::{Prism, PrismExt, Conversion};

use time::{Date, OffsetDateTime};

#[derive(Debug, Clone, Default)]
pub struct ExprToDateValues {
  _priv: (),
}

#[derive(Debug, Clone, Default)]
pub struct ExprToDatetimeValues {
  _priv: (),
}

pub fn expr_to_date() -> impl Prism<Expr, Date> + Clone {
  ExprToDateValues::new().composed(Conversion::new())
}

pub fn expr_to_offset_datetime() -> impl Prism<Expr, OffsetDateTime> + Clone {
  ExprToDatetimeValues::new().composed(Conversion::new())
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
