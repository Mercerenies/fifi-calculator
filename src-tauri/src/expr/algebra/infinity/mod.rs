
mod base;
mod prisms;
mod signed;
mod unbounded;

pub use base::InfiniteConstant;
pub use signed::{SignedInfinity, ExpectedSignedInfinityError};
pub use prisms::{ExprToInfinity, infinity_to_signed_infinity,
                 expr_to_signed_infinity, expr_to_unbounded_number};
pub use unbounded::{UnboundedNumber, IndeterminateFormError};

use crate::expr::Expr;
use crate::expr::number::{Number, ComplexLike};

use either::Either;
use num::{Zero, One};

use std::cmp::Ordering;

pub const INFINITY_NAME: &str = "inf";
pub const UNDIRECTED_INFINITY_NAME: &str = "uinf";
pub const NAN_NAME: &str = "nan";

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
