
use super::Interval;
use super::bound::Bounded;
use super::interval_type::IntervalType;
use crate::expr::{Expr, TryFromExprError};
use crate::util::prism::ErrorWithPayload;

use std::convert::TryFrom;

/// Equivalent to the [`Interval`] type but does not force its
/// structure into normal form. This is useful as the target of
/// prisms, since there is no data loss when storing information in
/// this structure.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RawInterval<T> {
  pub left: T,
  pub interval_type: IntervalType,
  pub right: T,
}

impl<T> RawInterval<T> {
  /// Constructs a new interval. This constructor does NOT normalize
  /// the interval.
  pub fn new(left: T, interval_type: IntervalType, right: T) -> Self {
    Self { left, interval_type, right }
  }

  /// Constructs a new interval from bounds. This constructor does NOT
  /// normalize the interval.
  pub fn from_bounds(left: Bounded<T>, right: Bounded<T>) -> Self {
    let interval_type = IntervalType::from_bounds(left.bound_type, right.bound_type);
    Self { left: left.scalar, interval_type, right: right.scalar }
  }

  pub fn normalize(self) -> Self where T: Ord + Default {
    Interval::from(self).into_raw()
  }

  pub fn into_bounds(self) -> (Bounded<T>, Bounded<T>) {
    let (left_bound, right_bound) = self.interval_type.into_bounds();
    (
      Bounded { scalar: self.left, bound_type: left_bound },
      Bounded { scalar: self.right, bound_type: right_bound },
    )
  }
}

impl<T> From<RawInterval<T>> for Expr
where T: Into<Expr> {
  fn from(interval: RawInterval<T>) -> Expr {
    Expr::call(interval.interval_type.name(), vec![interval.left.into(), interval.right.into()])
  }
}

impl<T: Ord + Default> From<RawInterval<T>> for Interval<T> {
  fn from(interval: RawInterval<T>) -> Self {
    Self::new(
      interval.left,
      interval.interval_type,
      interval.right,
    )
  }
}

fn try_from_expr_to_interval(expr: Expr) -> Result<RawInterval<Expr>, TryFromExprError> {
  const TYPE_NAME: &str = "RawInterval";
  if let Expr::Call(name, args) = expr {
    if args.len() == 2 {
      if let Ok(op) = IntervalType::parse(&name) {
        let [left, right] = args.try_into().unwrap(); // unwrap: Just checked the vec length.
        return Ok(RawInterval { left, interval_type: op, right });
      }
    }
    Err(TryFromExprError::new(TYPE_NAME, Expr::Call(name, args)))
  } else {
    Err(TryFromExprError::new(TYPE_NAME, expr))
  }
}

fn narrow_interval_type<T>(interval: RawInterval<Expr>) -> Result<RawInterval<T>, TryFromExprError>
where T: TryFrom<Expr>,
      Expr: From<T>,
      T::Error: ErrorWithPayload<Expr> {
  const TYPE_NAME: &str = "RawInterval";
  match T::try_from(interval.left) {
    Err(err) => Err(TryFromExprError::new(
      TYPE_NAME,
      RawInterval::new(err.recover_payload(), interval.interval_type, interval.right).into(),
    )),
    Ok(left) => {
      match T::try_from(interval.right) {
        Err(err) => Err(TryFromExprError::new(
          TYPE_NAME,
          RawInterval::new(left.into(), interval.interval_type, err.recover_payload()).into(),
        )),
        Ok(right) => Ok(RawInterval { left, interval_type: interval.interval_type, right }),
      }
    }
  }
}

impl<T> TryFrom<Expr> for RawInterval<T>
where T: TryFrom<Expr>,
      Expr: From<T>,
      T::Error: ErrorWithPayload<Expr> {
  type Error = TryFromExprError;

  fn try_from(expr: Expr) -> Result<Self, Self::Error> {
    let raw_interval = try_from_expr_to_interval(expr)?;
    narrow_interval_type(raw_interval)
  }
}

#[cfg(test)]
mod tests {
  use super::*;
  use crate::expr::number::Number;

  #[test]
  fn test_try_from_expr_for_raw_interval() {
    let expr = Expr::call("..", vec![Expr::from(0), Expr::from(1)]);
    assert_eq!(
      RawInterval::<Expr>::try_from(expr),
      Ok(RawInterval::<Expr>::new(Expr::from(0), IntervalType::Closed, Expr::from(1))),
    );
    let expr = Expr::call("..^", vec![Expr::call("foo", vec![]), Expr::from(2)]);
    assert_eq!(
      RawInterval::<Expr>::try_from(expr),
      Ok(RawInterval::<Expr>::new(Expr::call("foo", vec![]), IntervalType::RightOpen, Expr::from(2))),
    );
  }

  #[test]
  fn test_try_from_expr_for_raw_interval_failed() {
    let expr = Expr::call("foo", vec![Expr::from(0), Expr::from(1)]);
    assert_eq!(
      RawInterval::<Expr>::try_from(expr),
      Err(TryFromExprError::new(
        "RawInterval",
        Expr::call("foo", vec![Expr::from(0), Expr::from(1)])
      )),
    );
    let expr = Expr::call("..", vec![Expr::from(0), Expr::from(1), Expr::from(2)]);
    assert_eq!(
      RawInterval::<Expr>::try_from(expr),
      Err(TryFromExprError::new(
        "RawInterval",
        Expr::call("..", vec![Expr::from(0), Expr::from(1), Expr::from(2)])
      )),
    );
    let expr = Expr::from(0);
    assert_eq!(
      RawInterval::<Expr>::try_from(expr),
      Err(TryFromExprError::new(
        "RawInterval",
        Expr::from(0),
      )),
    );
  }

  #[test]
  fn test_try_from_expr_for_raw_interval_number() {
    let expr = Expr::call("..", vec![Expr::from(0), Expr::from(1)]);
    assert_eq!(
      RawInterval::try_from(expr),
      Ok(RawInterval::new(Number::from(0), IntervalType::Closed, Number::from(1))),
    );
  }

  #[test]
  fn test_try_from_expr_for_interval_with_non_literal_arg() {
    let expr = Expr::call("..^", vec![Expr::call("foo", vec![]), Expr::from(2)]);
    assert_eq!(
      RawInterval::<Number>::try_from(expr),
      Err(TryFromExprError::new(
        "RawInterval",
        Expr::call("..^", vec![Expr::call("foo", vec![]), Expr::from(2)])
      )),
    );
  }

  #[test]
  fn test_try_from_expr_for_interval_failed() {
    let expr = Expr::call("foo", vec![Expr::from(0), Expr::from(1)]);
    assert_eq!(
      RawInterval::<Number>::try_from(expr),
      Err(TryFromExprError::new(
        "RawInterval",
        Expr::call("foo", vec![Expr::from(0), Expr::from(1)])
      )),
    );
    let expr = Expr::call("..", vec![Expr::from(0), Expr::from(1), Expr::from(2)]);
    assert_eq!(
      RawInterval::<Number>::try_from(expr),
      Err(TryFromExprError::new(
        "RawInterval",
        Expr::call("..", vec![Expr::from(0), Expr::from(1), Expr::from(2)])
      )),
    );
    let expr = Expr::from(0);
    assert_eq!(
      RawInterval::<Number>::try_from(expr),
      Err(TryFromExprError::new(
        "RawInterval",
        Expr::from(0),
      )),
    );
  }
}
