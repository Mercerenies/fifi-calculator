
//! Miscellaneous Boolean predicates on the [`Expr`] type.

use super::Expr;
use super::atom::Atom;
use super::number::{ComplexNumber, Quaternion};
use super::vector::borrowed::BorrowedVector;
use super::interval::IntervalType;

pub use super::algebra::infinity::{is_infinite_constant, is_signed_infinite_constant};

/// Returns true if `expr` is a real [`Number`](super::number::Number)
/// value.
pub fn is_real(expr: &Expr) -> bool {
  matches!(expr, Expr::Atom(Atom::Number(_)))
}

/// Returns true if `expr` is a real number or a signed infinity.
pub fn is_unbounded_number(expr: &Expr) -> bool {
  is_real(expr) || is_signed_infinite_constant(expr)
}

/// Returns true if `expr` is a real or complex number literal.
pub fn is_complex(expr: &Expr) -> bool {
  if is_real(expr) {
    return true;
  }
  if let Expr::Call(f, args) = expr {
    if f == ComplexNumber::FUNCTION_NAME && args.len() == 2 {
      return true;
    }
  }
  false
}

/// Returns true if `expr` is a real or complex number literal, or an infinity constant.
pub fn is_complex_or_inf(expr: &Expr) -> bool {
  is_complex(expr) || is_infinite_constant(expr)
}

/// Returns true if `expr` is a real, complex, or quaternion literal.
pub fn is_quaternion(expr: &Expr) -> bool {
  if is_complex(expr) {
    return true;
  }
  if let Expr::Call(f, args) = expr {
    if f == Quaternion::FUNCTION_NAME && args.len() == 4 {
      return true;
    }
  }
  false
}

/// Returns true if `expr` is an [`Expr::Call`] whose head is the
/// vector function.
pub fn is_vector(expr: &Expr) -> bool {
  BorrowedVector::parse(expr).is_ok()
}

/// Returns true if `expr` is a string atom.
pub fn is_string(expr: &Expr) -> bool {
  matches!(expr, Expr::Atom(Atom::String(_)))
}

/// Returns true if `expr` is a two-argument interval call, where each
/// argument is an unbounded number (per [`is_unbounded_number`]). The
/// type of interval (i.e. open, closed, or half open) is irrelevant.
pub fn is_unbounded_interval(expr: &Expr) -> bool {
  let Expr::Call(f, args) = expr else { return false; };
  if !IntervalType::is_interval_type(f) {
    return false;
  }
  args.len() == 2 && args.iter().all(|x| is_unbounded_number(x))
}

/// Returns true if `expr` is either an unbounded interval (per
/// [`is_unbounded_interval`]) or an unbounded number (per
/// [`is_unbounded_number`]).
pub fn is_unbounded_interval_like(expr: &Expr) -> bool {
  is_unbounded_interval(expr) || is_unbounded_number(expr)
}
