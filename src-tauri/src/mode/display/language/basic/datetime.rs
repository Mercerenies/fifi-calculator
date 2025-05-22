
//! Pretty-printing functionality for datetime objects.

use crate::expr::Expr;
use crate::expr::datetime::DateTime;
use crate::expr::datetime::prisms::expr_to_datetime;
use crate::util::prism::Prism;

use time::error::Format;
use time::macros::format_description;
use thiserror::Error;
use html_escape::encode_safe;

use std::io::{self, Write};
use std::error::{Error as StdError};
use std::str::from_utf8;
use std::fmt;
use std::borrow::Cow;

#[derive(Debug, Error)]
#[non_exhaustive]
pub enum DatetimeWriteError {
  #[error("{0}")]
  Format(#[from] Format),
  #[error("Invalid datetime call")]
  BadDatetimeCall,
}

impl DatetimeWriteError {
  fn custom(err: impl StdError + Send + Sync + 'static) -> Self {
    let io_err = io::Error::new(io::ErrorKind::Other, Box::new(err));
    Self::Format(io_err.into())
  }
}

impl From<io::Error> for DatetimeWriteError {
  fn from(err: io::Error) -> Self {
    DatetimeWriteError::Format(err.into())
  }
}

pub fn write_datetime<W: Write>(out: &mut W, datetime: &DateTime) -> Result<(), Format> {
  if datetime.has_time() {
    let fmt = if datetime.second() == 0 && datetime.microsecond() == 0 {
      format_description!("[hour padding:none repr:12]:[minute][period case:lower] [weekday repr:short] [month repr:short] [day padding:none], [year]")
    } else if datetime.microsecond() == 0 {
      format_description!("[hour padding:none repr:12]:[minute]:[second][period case:lower] [weekday repr:short] [month repr:short] [day padding:none], [year]")
    } else {
      format_description!("[hour padding:none repr:12]:[minute]:[second].[subsecond][period case:lower] [weekday repr:short] [month repr:short] [day padding:none], [year]")
    };
    datetime.to_offset_date_time().format_into(out, fmt)?;
    write!(out, " {}", datetime.timezone_offset())?;
  } else {
    let fmt = format_description!("[weekday repr:short] [month repr:short] [day padding:none], [year]");
    datetime.date().format_into(out, fmt)?;
  }
  Ok(())
}

/// Writes an expression, which must be a `datetime` call of [valid
/// arity](crate::expr::datetime::DATETIME_ARITIES), as a datetime. If
/// the expression is not a valid `datetime` call or contains values
/// out of range, then nothing is printed, and an appropriate error is
/// return.
pub fn write_datetime_expr<W: Write>(out: &mut W, expr: Expr) -> Result<(), DatetimeWriteError> {
  // TODO Can we get this function to take &Expr, rather than
  // requiring a clone at the call site?
  let Ok(datetime) = expr_to_datetime().narrow_type(expr) else {
    return Err(DatetimeWriteError::BadDatetimeCall);
  };
  write!(out, "#<")?;
  write_datetime(out, &datetime)?;
  write!(out, ">")?;
  Ok(())
}

pub fn write_datetime_expr_fmt<W: fmt::Write>(out: &mut W, expr: Expr, escape_html: bool) -> Result<(), DatetimeWriteError> {
  // The time crate uses io::Write, which I think is incorrect since
  // it only writes UTF-8 streams. But I want to write to String. So
  // we write to a Vec<u8> and then convert.
  let mut buf = Vec::<u8>::with_capacity(32);
  write_datetime_expr(&mut buf, expr)?;
  let s = from_utf8(&buf).map_err(DatetimeWriteError::custom)?;
  let s = if escape_html { encode_safe(&s) } else { Cow::Borrowed(s) };
  write!(out, "{}", s).map_err(DatetimeWriteError::custom)?;
  Ok(())
}
