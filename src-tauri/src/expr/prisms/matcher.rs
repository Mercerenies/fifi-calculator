
//! API for a pattern-matching-like prism generator for expressions.

use crate::expr::{Expr, TryFromExprError};
use crate::util::prism::Conversion;

use thiserror::Error;

use std::convert::{AsRef, TryFrom};
use std::marker::PhantomData;
use std::fmt::{self, Display, Debug, Formatter};

/// A `MatcherSpec` is a type-level object used to specify the desired
/// top-level shape of an expression. None of the functions or
/// constants here depend on a `self` parameter, so `MatcherSpec`
/// objects should be singleton types and are only used for the
/// type-level information.
pub trait MatcherSpec {
  /// The name of the expected function.
  const FUNCTION_NAME: &'static str;

  /// The minimum arity of the expected function, inclusive.
  const MIN_ARITY: usize;

  /// The maximum arity of the expected function, inclusive. For
  /// functions with no maximum arity, this function should return
  /// `usize::MAX`.
  const MAX_ARITY: usize;

  /// A [`Prism`](crate::util::prism::Prism) which parses an
  /// expression using this specification. The prism succeeds if the
  /// input expression is an `Expr::Call` whose function name is
  /// `Self::FUNCTION_NAME` and whose arity is between
  /// `Self::MIN_ARITY` and `Self::MAX_ARITY` inclusive.
  ///
  /// Implementors shall NOT override this method.
  fn prism() -> Conversion<Expr, MatchedExpr<Self>>
  where Self: Sized {
    Conversion::new()
  }
}

/// A `MatchedExpr<S>` is an `Expr` which matches the specification
/// given by the phantom type argument `S`.
///
/// This type is essentially a newtype wrapper around an `Expr`. It
/// can be converted to an `Expr` via `From` / `Into`, and an `Expr`
/// can be parsed into this type using `TryFrom` / `TryInto`.
/// Alternatively, the prism [`MatcherSpec::prism`] can be used to do
/// the same conversions.
pub struct MatchedExpr<S: MatcherSpec> {
  // This expression MUST always be a Expr::Call. We store it as an
  // Expr to allow AsRef to work. All constructors verify this
  // precondition.
  inner_expr: Expr,
  _phantom: PhantomData<fn() -> S>,
}

#[derive(Debug, Clone, Error)]
pub struct ArgsLengthError {
  args: Vec<Expr>,
  min_expected: usize,
  max_expected: usize,
}

impl<S: MatcherSpec> MatchedExpr<S> {
  pub fn new(args: Vec<Expr>) -> Result<Self, ArgsLengthError> {
    if args.len() < S::MIN_ARITY || args.len() > S::MAX_ARITY {
      Err(ArgsLengthError {
        args,
        min_expected: S::MIN_ARITY,
        max_expected: S::MAX_ARITY,
      })
    } else {
      Ok(Self {
        inner_expr: Expr::call(S::FUNCTION_NAME, args),
        _phantom: PhantomData,
      })
    }
  }

  /// Constructs a `MatchedExpr` with the given arity and matching the
  /// given spec. Panics if the number of arguments is incorrect.
  pub fn new_or_panic(args: Vec<Expr>) -> Self {
    match Self::new(args) {
      Ok(m) => m,
      Err(e) => panic!("{}", e),
    }
  }

  /// Borrows the arguments to the matched expression.
  pub fn args(&self) -> &[Expr] {
    let Expr::Call(_, args) = &self.inner_expr else {
      panic!("MatchedExpr::inner_expr must be an Expr::Call");
    };
    args
  }

  /// Returns the arguments to the matched expression.
  pub fn into_args(self) -> Vec<Expr> {
    let Expr::Call(_, args) = self.inner_expr else {
      panic!("MatchedExpr::inner_expr must be an Expr::Call");
    };
    args
  }
}

impl<S: MatcherSpec> From<MatchedExpr<S>> for Expr {
  fn from(m: MatchedExpr<S>) -> Expr {
    m.inner_expr
  }
}

impl<S: MatcherSpec> Clone for MatchedExpr<S> {
  fn clone(&self) -> Self {
    MatchedExpr {
      inner_expr: self.inner_expr.clone(),
      _phantom: PhantomData,
    }
  }
}

impl<S: MatcherSpec> AsRef<Expr> for MatchedExpr<S> {
  fn as_ref(&self) -> &Expr {
    &self.inner_expr
  }
}

impl<S: MatcherSpec> PartialEq for MatchedExpr<S> {
  fn eq(&self, other: &Self) -> bool {
    self.inner_expr == other.inner_expr
  }
}

impl Display for ArgsLengthError {
  fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
    if self.min_expected == self.max_expected {
      write!(f, "Incorrect number of arguments: expected {}, actual {}",
             self.min_expected, self.args.len())
    } else {
      write!(f, "Incorrect number of arguments: expected {} to {}, actual {}",
             self.min_expected, self.max_expected, self.args.len())
    }
  }
}

// Manual impl of Debug so that we don't require `S: Debug`.
impl<S: MatcherSpec> Debug for MatchedExpr<S> {
  fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
    f.debug_struct("MatchedExpr")
      .field("inner_expr", &self.inner_expr)
      .field("_phantom", &self._phantom)
      .finish()
  }
}

impl<S: MatcherSpec> TryFrom<Expr> for MatchedExpr<S> {
  type Error = TryFromExprError;

  fn try_from(expr: Expr) -> Result<Self, Self::Error> {
    if let Expr::Call(function_name, args) = expr {
      if function_name == S::FUNCTION_NAME {
        Self::new(args).map_err(|err| {
          TryFromExprError::new(S::FUNCTION_NAME, Expr::Call(function_name, err.args))
        })
      } else {
        Err(TryFromExprError::new(S::FUNCTION_NAME, Expr::Call(function_name, args)))
      }
    } else {
      Err(TryFromExprError::new(S::FUNCTION_NAME, expr))
    }
  }
}

#[cfg(test)]
mod tests {
  use super::*;
  use crate::util::prism::Prism;

  struct TestMatcherSpec;

  impl MatcherSpec for TestMatcherSpec {
    const FUNCTION_NAME: &'static str = "test";
    const MIN_ARITY: usize = 1;
    const MAX_ARITY: usize = 2;
  }

  #[test]
  fn test_matcher_spec_against_atoms() {
    assert_eq!(TestMatcherSpec::prism().narrow_type(Expr::from(0)), Err(Expr::from(0)));
    assert_eq!(TestMatcherSpec::prism().narrow_type(Expr::var("x").unwrap()), Err(Expr::var("x").unwrap()));
  }

  #[test]
  fn test_matcher_spec_against_wrong_function_name() {
    let expr = Expr::call("wrong_function", vec![]);
    assert_eq!(TestMatcherSpec::prism().narrow_type(expr.clone()), Err(expr));
    let expr = Expr::call("wrong_function", vec![Expr::from(0)]);
    assert_eq!(TestMatcherSpec::prism().narrow_type(expr.clone()), Err(expr));
    let expr = Expr::call("wrong_function", vec![Expr::from(0), Expr::from(0)]);
    assert_eq!(TestMatcherSpec::prism().narrow_type(expr.clone()), Err(expr));
    let expr = Expr::call("wrong_function", vec![Expr::from(0), Expr::from(0), Expr::from(0)]);
    assert_eq!(TestMatcherSpec::prism().narrow_type(expr.clone()), Err(expr));
  }

  #[test]
  fn test_matcher_spec_against_wrong_function_arity() {
    let expr = Expr::call("test", vec![]);
    assert_eq!(TestMatcherSpec::prism().narrow_type(expr.clone()), Err(expr));
    let expr = Expr::call("test", vec![Expr::from(0), Expr::from(0), Expr::from(0)]);
    assert_eq!(TestMatcherSpec::prism().narrow_type(expr.clone()), Err(expr));
  }

  #[test]
  fn test_matcher_spec_successful() {
    let expr = Expr::call("test", vec![Expr::from(0)]);
    let matched_expr = TestMatcherSpec::prism().narrow_type(expr.clone()).unwrap();
    assert_eq!(Expr::from(matched_expr), expr);
    let expr = Expr::call("test", vec![Expr::from(0), Expr::from(9)]);
    let matched_expr = TestMatcherSpec::prism().narrow_type(expr.clone()).unwrap();
    assert_eq!(Expr::from(matched_expr), expr);
    let expr = Expr::call("test", vec![Expr::from(0), Expr::call("inner_function_call", vec![])]);
    let matched_expr = TestMatcherSpec::prism().narrow_type(expr.clone()).unwrap();
    assert_eq!(Expr::from(matched_expr), expr);
  }

  #[test]
  fn test_widen_type() {
    let matched_expr = MatchedExpr {
      inner_expr: Expr::call("test", vec![Expr::from(0), Expr::from(10)]),
      _phantom: PhantomData,
    };
    assert_eq!(
      TestMatcherSpec::prism().widen_type(matched_expr),
      Expr::call("test", vec![Expr::from(0), Expr::from(10)]),
    )
  }
}
