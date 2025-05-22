
use thiserror::Error;

use std::num::ParseIntError;

#[derive(Debug, Clone, Error, PartialEq, Eq)]
#[non_exhaustive]
pub enum DatetimeParseError {
  #[error("{0}")]
  ParseIntError(#[from] ParseIntError),
  #[error("'noon' and 'midnight' can only be applied to the time 12:00")]
  MisappliedNoonOrMid,
  #[error("Applied AM or PM modifier to 24-hour time")]
  PeriodOn24HourTime,
  #[error("Field '{field_name}' out of range")]
  DatetimeConstructionError { field_name: &'static str },
  #[error("Malformed year field")]
  MalformedYearField,
  #[error("Unexpected token '{token}'")]
  UnexpectedToken { token: String },
}
