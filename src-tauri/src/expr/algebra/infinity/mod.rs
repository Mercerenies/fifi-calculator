
mod prisms;

pub use prisms::{ExprToInfinity, infinity_to_signed_infinity, expr_to_signed_infinity, expr_to_unbounded_number};

use crate::expr::Expr;
use crate::expr::number::{Number, ComplexLike};
use crate::util::prism::ErrorWithPayload;

use either::Either;
use num::{Zero, One};
use thiserror::Error;

use std::fmt::{self, Display, Formatter};
use std::ops::{Add, Sub, Mul, MulAssign, Div, Neg};
use std::cmp::Ordering;
use std::convert::TryFrom;

pub const INFINITY_NAME: &str = "inf";
pub const UNDIRECTED_INFINITY_NAME: &str = "uinf";
pub const NAN_NAME: &str = "nan";

/// A limit value on the bounds of the usual real line (or complex
/// plane).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum InfiniteConstant {
  /// Positive infinity, greater than any finite real value.
  PosInfinity,
  /// Negative infinity, less than any finite real value.
  NegInfinity,
  /// An infinite quantity whose direction cannot be determined.
  UndirInfinity,
  /// An unknown quantity, usually resulting from an undeterminate
  /// form.
  ///
  /// Note: Unlike the IEEE 754 constant of the same name, this
  /// constant DOES compare equal to itself. The type
  /// [`InfiniteConstant`] correctly implements both `PartialEq` and
  /// `Eq`.
  NotANumber,
}

/// An infinity value with a known sign.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum SignedInfinity {
  NegInfinity,
  PosInfinity,
}

/// Either a finite real value or a signed infinity.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum UnboundedNumber {
  Finite(Number),
  Infinite(SignedInfinity),
}

#[derive(Debug, Clone, Error)]
#[error("Expected signed infinity")]
pub struct ExpectedSignedInfinityError {
  original_constant: InfiniteConstant,
}

impl InfiniteConstant {
  pub const ALL: [InfiniteConstant; 4] = [
    InfiniteConstant::PosInfinity,
    InfiniteConstant::NegInfinity,
    InfiniteConstant::UndirInfinity,
    InfiniteConstant::NotANumber,
  ];

  pub fn abs(&self) -> InfiniteConstant {
    if self == &InfiniteConstant::NotANumber {
      InfiniteConstant::NotANumber
    } else {
      InfiniteConstant::PosInfinity
    }
  }
}

impl From<SignedInfinity> for InfiniteConstant {
  fn from(s: SignedInfinity) -> InfiniteConstant {
    match s {
      SignedInfinity::NegInfinity => InfiniteConstant::NegInfinity,
      SignedInfinity::PosInfinity => InfiniteConstant::PosInfinity,
    }
  }
}

impl TryFrom<InfiniteConstant> for SignedInfinity {
  type Error = ExpectedSignedInfinityError;

  fn try_from(c: InfiniteConstant) -> Result<SignedInfinity, ExpectedSignedInfinityError> {
    match c {
      InfiniteConstant::NegInfinity => Ok(SignedInfinity::NegInfinity),
      InfiniteConstant::PosInfinity => Ok(SignedInfinity::PosInfinity),
      _ => Err(ExpectedSignedInfinityError { original_constant: c }),
    }
  }
}

impl ErrorWithPayload<InfiniteConstant> for ExpectedSignedInfinityError {
  fn recover_payload(self) -> InfiniteConstant {
    self.original_constant
  }
}

impl PartialEq<Number> for SignedInfinity {
  fn eq(&self, _: &Number) -> bool {
    false
  }
}

impl PartialOrd<Number> for SignedInfinity {
  fn partial_cmp(&self, _other: &Number) -> Option<Ordering> {
    match self {
      SignedInfinity::NegInfinity => Some(Ordering::Less),
      SignedInfinity::PosInfinity => Some(Ordering::Greater),
    }
  }
}

impl PartialEq<SignedInfinity> for Number {
  fn eq(&self, _: &SignedInfinity) -> bool {
    false
  }
}

impl PartialOrd<SignedInfinity> for Number {
  fn partial_cmp(&self, other: &SignedInfinity) -> Option<Ordering> {
    other.partial_cmp(self).map(Ordering::reverse)
  }
}

impl PartialOrd for UnboundedNumber {
  fn partial_cmp(&self, other: &UnboundedNumber) -> Option<Ordering> {
    Some(self.cmp(other))
  }
}

impl Ord for UnboundedNumber {
  fn cmp(&self, other: &Self) -> Ordering {
    match (self, other) {
      (UnboundedNumber::Finite(a), UnboundedNumber::Finite(b)) => a.cmp(b),
      (UnboundedNumber::Finite(a), UnboundedNumber::Infinite(b)) => a.partial_cmp(b).unwrap(),
      (UnboundedNumber::Infinite(a), UnboundedNumber::Finite(b)) => a.partial_cmp(b).unwrap(),
      (UnboundedNumber::Infinite(a), UnboundedNumber::Infinite(b)) => a.partial_cmp(b).unwrap(),
    }
  }
}


pub fn is_infinite_constant(expr: &Expr) -> bool {
  InfiniteConstant::ALL.iter().any(|c| &Expr::from(c) == expr)
}

// TODO: Nightmarishly long function; clean up!
pub fn multiply_infinities(args: Vec<Either<ComplexLike, InfiniteConstant>>) -> Expr {
  // If all quantities are finite, then just do regular
  // multiplication.
  if args.iter().all(|arg| arg.is_left()) {
    let complex_product = args.into_iter().fold(ComplexLike::one(), |acc, arg| {
      acc * arg.unwrap_left()
    });
    return Expr::from(complex_product);
  }

  // Track the direction and magnitude of the infinite value
  // separately.
  let mut scalar = ComplexLike::one();
  let mut infinity = InfiniteConstant::PosInfinity;
  for arg in args {
    match arg {
      Either::Left(z) => {
        scalar *= z;
      }
      Either::Right(InfiniteConstant::NegInfinity) => {
        // Move the negative sign to the scalar part.
        scalar = - scalar;
      }
      Either::Right(inf) => {
        infinity *= inf;
      }
    }
  }

  // If the scalar is zero, then we have zero times infinity, which is
  // NaN.
  if scalar.is_zero() {
    return Expr::from(InfiniteConstant::NotANumber);
  }

  // If infinity is undirected or NaN, then ignore the scalar.
  if infinity == InfiniteConstant::UndirInfinity || infinity == InfiniteConstant::NotANumber {
    return Expr::from(infinity);
  }

  match scalar {
    ComplexLike::Real(r) => {
      // If the scalar is a real number, then we have a simple
      // infinite result.
      match r.cmp(&Number::zero()) {
        Ordering::Greater => Expr::from(InfiniteConstant::PosInfinity),
        Ordering::Less => Expr::from(InfiniteConstant::NegInfinity),
        Ordering::Equal => unreachable!(),
      }
    }
    ComplexLike::Complex(z) => {
      // Otherwise, we have a complex-valued infinity.
      assert!(infinity == InfiniteConstant::PosInfinity, "Expected +inf in infinity multiplication");
      assert!(!z.is_zero(), "Expected non-zero scalar in complex infinity multiplication");
      Expr::call("*", vec![
        Expr::from(z.signum()),
        Expr::from(InfiniteConstant::PosInfinity),
      ])
    }
  }
}

pub fn infinite_pow(left: InfiniteConstant, right: InfiniteConstant) -> Expr {
  use InfiniteConstant::*;
  match (left, right) {
    (NotANumber, _) | (_, NotANumber) => Expr::from(NotANumber),
    (_, UndirInfinity) => Expr::from(NotANumber),
    (_, NegInfinity) => Expr::zero(),
    (left, PosInfinity) => Expr::from(left),
  }
}

impl<'a> From<&'a InfiniteConstant> for Expr {
  fn from(c: &'a InfiniteConstant) -> Self {
    match c {
      InfiniteConstant::PosInfinity => Expr::var(INFINITY_NAME).unwrap(),
      InfiniteConstant::NegInfinity => Expr::call("negate", vec![Expr::var(INFINITY_NAME).unwrap()]),
      InfiniteConstant::UndirInfinity => Expr::var(UNDIRECTED_INFINITY_NAME).unwrap(),
      InfiniteConstant::NotANumber => Expr::var(NAN_NAME).unwrap(),
    }
  }
}

impl From<InfiniteConstant> for Expr {
  fn from(c: InfiniteConstant) -> Self {
    Expr::from(&c)
  }
}

impl From<SignedInfinity> for Expr {
  fn from(c: SignedInfinity) -> Self {
    Expr::from(InfiniteConstant::from(c))
  }
}

impl From<UnboundedNumber> for Expr {
  fn from(c: UnboundedNumber) -> Self {
    match c {
      UnboundedNumber::Finite(c) => Expr::from(c),
      UnboundedNumber::Infinite(c) => Expr::from(c),
    }
  }
}

impl Display for InfiniteConstant {
  fn fmt(&self, f: &mut Formatter) -> fmt::Result {
    match self {
      InfiniteConstant::PosInfinity => write!(f, "inf"),
      InfiniteConstant::NegInfinity => write!(f, "-inf"),
      InfiniteConstant::UndirInfinity => write!(f, "uinf"),
      InfiniteConstant::NotANumber => write!(f, "nan"),
    }
  }
}

impl Add for InfiniteConstant {
  type Output = InfiniteConstant;

  fn add(self, other: InfiniteConstant) -> InfiniteConstant {
    use InfiniteConstant::*;
    match (self, other) {
      (NotANumber, _) | (_, NotANumber) => NotANumber,
      (UndirInfinity, _) | (_, UndirInfinity) => UndirInfinity,
      (PosInfinity, NegInfinity) | (NegInfinity, PosInfinity) => NotANumber,
      (PosInfinity, PosInfinity) => PosInfinity,
      (NegInfinity, NegInfinity) => PosInfinity,
    }
  }
}

impl Neg for InfiniteConstant {
  type Output = InfiniteConstant;

  fn neg(self) -> InfiniteConstant {
    use InfiniteConstant::*;
    match self {
      NotANumber => NotANumber,
      PosInfinity => NegInfinity,
      NegInfinity => PosInfinity,
      UndirInfinity => UndirInfinity,
    }
  }
}

impl Sub for InfiniteConstant {
  type Output = InfiniteConstant;

  fn sub(self, other: InfiniteConstant) -> InfiniteConstant {
    self + -other
  }
}

impl Mul for InfiniteConstant {
  type Output = InfiniteConstant;

  fn mul(self, other: InfiniteConstant) -> InfiniteConstant {
    use InfiniteConstant::*;
    match (self, other) {
      (NotANumber, _) | (_, NotANumber) => NotANumber,
      (UndirInfinity, _) | (_, UndirInfinity) => UndirInfinity,
      (PosInfinity, NegInfinity) | (NegInfinity, PosInfinity) => NegInfinity,
      (PosInfinity, PosInfinity) | (NegInfinity, NegInfinity) => PosInfinity,
    }
  }
}

impl MulAssign for InfiniteConstant {
  fn mul_assign(&mut self, other: InfiniteConstant) {
    *self = *self * other
  }
}

/// Trivial implementation of `Div`. Since we can't compare the
/// relative magnitudes of two infinities, we always get `NotANumber`.
impl Div for InfiniteConstant {
  type Output = InfiniteConstant;

  fn div(self, _: InfiniteConstant) -> InfiniteConstant {
    InfiniteConstant::NotANumber
  }
}

#[cfg(test)]
mod tests {
  use super::*;
  use crate::expr::number::ComplexNumber;

  #[test]
  fn test_empty_multiplication() {
    let args: Vec<Either<ComplexLike, InfiniteConstant>> = Vec::new();
    let result = multiply_infinities(args);
    assert_eq!(result, Expr::one());
  }

  #[test]
  fn test_ordinary_finite_multiplication() {
    let args: Vec<Either<ComplexLike, InfiniteConstant>> = vec![
      Either::Left(ComplexLike::from(10)),
      Either::Left(ComplexLike::from(3)),
      Either::Left(ComplexLike::from(-1)),
    ];
    let result = multiply_infinities(args);
    assert_eq!(result, Expr::from(-30));
  }

  #[test]
  fn test_single_infinity_multiplication() {
    let args: Vec<Either<ComplexLike, InfiniteConstant>> = vec![
      Either::Right(InfiniteConstant::PosInfinity),
    ];
    let result = multiply_infinities(args);
    assert_eq!(result, Expr::from(InfiniteConstant::PosInfinity));

    let args: Vec<Either<ComplexLike, InfiniteConstant>> = vec![
      Either::Right(InfiniteConstant::NegInfinity),
    ];
    let result = multiply_infinities(args);
    assert_eq!(result, Expr::from(InfiniteConstant::NegInfinity));

    let args: Vec<Either<ComplexLike, InfiniteConstant>> = vec![
      Either::Right(InfiniteConstant::UndirInfinity),
    ];
    let result = multiply_infinities(args);
    assert_eq!(result, Expr::from(InfiniteConstant::UndirInfinity));

    let args: Vec<Either<ComplexLike, InfiniteConstant>> = vec![
      Either::Right(InfiniteConstant::NotANumber),
    ];
    let result = multiply_infinities(args);
    assert_eq!(result, Expr::from(InfiniteConstant::NotANumber));
  }

  #[test]
  fn test_infinity_multiplication() {
    let args: Vec<Either<ComplexLike, InfiniteConstant>> = vec![
      Either::Right(InfiniteConstant::PosInfinity),
      Either::Right(InfiniteConstant::NegInfinity),
    ];
    let result = multiply_infinities(args);
    assert_eq!(result, Expr::from(InfiniteConstant::NegInfinity));

    let args: Vec<Either<ComplexLike, InfiniteConstant>> = vec![
      Either::Right(InfiniteConstant::NegInfinity),
      Either::Right(InfiniteConstant::NegInfinity),
      Either::Right(InfiniteConstant::NegInfinity),
    ];
    let result = multiply_infinities(args);
    assert_eq!(result, Expr::from(InfiniteConstant::NegInfinity));

    let args: Vec<Either<ComplexLike, InfiniteConstant>> = vec![
      Either::Right(InfiniteConstant::PosInfinity),
      Either::Right(InfiniteConstant::UndirInfinity),
      Either::Right(InfiniteConstant::PosInfinity),
    ];
    let result = multiply_infinities(args);
    assert_eq!(result, Expr::from(InfiniteConstant::UndirInfinity));

    let args: Vec<Either<ComplexLike, InfiniteConstant>> = vec![
      Either::Right(InfiniteConstant::NotANumber),
      Either::Right(InfiniteConstant::UndirInfinity),
      Either::Right(InfiniteConstant::PosInfinity),
    ];
    let result = multiply_infinities(args);
    assert_eq!(result, Expr::from(InfiniteConstant::NotANumber));
  }

  #[test]
  fn test_infinity_multiplication_by_real_numbers() {
    let args: Vec<Either<ComplexLike, InfiniteConstant>> = vec![
      Either::Right(InfiniteConstant::NegInfinity),
      Either::Left(ComplexLike::from(3)),
      Either::Left(ComplexLike::from(-9)),
    ];
    let result = multiply_infinities(args);
    assert_eq!(result, Expr::from(InfiniteConstant::PosInfinity));

    let args: Vec<Either<ComplexLike, InfiniteConstant>> = vec![
      Either::Right(InfiniteConstant::NegInfinity),
      Either::Left(ComplexLike::from(-3)),
      Either::Left(ComplexLike::from(-9)),
    ];
    let result = multiply_infinities(args);
    assert_eq!(result, Expr::from(InfiniteConstant::NegInfinity));
  }

  #[test]
  fn test_infinity_multiplication_by_zero() {
    let args: Vec<Either<ComplexLike, InfiniteConstant>> = vec![
      Either::Right(InfiniteConstant::NegInfinity),
      Either::Left(ComplexLike::from(3)),
      Either::Left(ComplexLike::from(0)),
    ];
    let result = multiply_infinities(args);
    assert_eq!(result, Expr::from(InfiniteConstant::NotANumber));
  }

  #[test]
  fn test_mixed_infinity_multiplication_with_nan() {
    let args: Vec<Either<ComplexLike, InfiniteConstant>> = vec![
      Either::Right(InfiniteConstant::NotANumber),
      Either::Left(ComplexLike::from(3)),
      Either::Left(ComplexLike::from(4)),
    ];
    let result = multiply_infinities(args);
    assert_eq!(result, Expr::from(InfiniteConstant::NotANumber));
  }

  #[test]
  fn test_mixed_infinity_multiplication_with_undirected_inf() {
    let args: Vec<Either<ComplexLike, InfiniteConstant>> = vec![
      Either::Left(ComplexLike::from(3)),
      Either::Left(ComplexLike::from(4)),
      Either::Right(InfiniteConstant::UndirInfinity),
    ];
    let result = multiply_infinities(args);
    assert_eq!(result, Expr::from(InfiniteConstant::UndirInfinity));
  }

  #[test]
  fn test_mixed_infinity_multiplication_with_undirected_inf_and_nan() {
    let args: Vec<Either<ComplexLike, InfiniteConstant>> = vec![
      Either::Left(ComplexLike::from(3)),
      Either::Left(ComplexLike::from(4)),
      Either::Right(InfiniteConstant::UndirInfinity),
      Either::Right(InfiniteConstant::NotANumber),
    ];
    let result = multiply_infinities(args);
    assert_eq!(result, Expr::from(InfiniteConstant::NotANumber));
  }

  #[test]
  fn test_complex_infinity_multiplication() {
    let args: Vec<Either<ComplexLike, InfiniteConstant>> = vec![
      Either::Left(ComplexLike::Complex(ComplexNumber::new(3, 4))),
      Either::Right(InfiniteConstant::PosInfinity),
    ];
    let result = multiply_infinities(args);
    assert_eq!(result, Expr::call("*", vec![
      Expr::from(ComplexNumber::new(0.6, 0.8)),
      Expr::from(InfiniteConstant::PosInfinity),
    ]));

    let args: Vec<Either<ComplexLike, InfiniteConstant>> = vec![
      Either::Left(ComplexLike::Complex(ComplexNumber::new(3, 4))),
      Either::Right(InfiniteConstant::NegInfinity),
    ];
    let result = multiply_infinities(args);
    assert_eq!(result, Expr::call("*", vec![
      Expr::from(ComplexNumber::new(-0.6, -0.8)),
      Expr::from(InfiniteConstant::PosInfinity),
    ]));
  }
}
