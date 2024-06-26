
//! Every command accepts a list of zero or more arguments, which are
//! passed over the Tauri boundary as strings.
//!
//! Our system expects certain things of these arguments. This module
//! provides schemas to validate those assumptions. Note that a
//! violation of these assumptions should be treated as a bug in the
//! program, similar to a type error. For instance, if a command
//! expects a variable name, then the frontend is responsible for
//! validating that the user input is in fact a valid variable name
//! _before_ passing it to the schema. If a bad variable name makes it
//! as far as the schema, then that's a bug in this application.

use crate::util::prism::{Prism, Identity};

use thiserror::Error;

use std::error::{Error as StdError};
use std::fmt::{self, Display, Formatter};
use std::marker::PhantomData;

/// An implementor of `ArgumentSchema` is capable of validating and
/// coercing a `Vec<String>` into some output data type.
pub trait ArgumentSchema {
  type Output;

  fn validate(&self, args: Vec<String>) -> Result<Self::Output, ArgumentSchemaError>;
}

#[derive(Clone, Debug, Default)]
pub struct NullaryArgumentSchema {
  _priv: (),
}

#[derive(Clone, Debug)]
pub struct UnaryArgumentSchema<P, T> {
  expected: String,
  prism: P,
  _phantom: PhantomData<fn() -> T>,
}

#[derive(Clone, Debug)]
pub struct BinaryArgumentSchema<P1, T1, P2, T2> {
  first_expected: String,
  first_prism: P1,
  second_expected: String,
  second_prism: P2,
  _phantom: PhantomData<fn() -> (T1, T2)>,
}

/// An error occurring during validation of an argument list schema.
#[derive(Debug)]
pub struct ArgumentSchemaError {
  body: Box<dyn StdError + Send + Sync + 'static>,
}

/// This error type displays only as a generic error message, intended
/// to be shown to the user in case of schema failure. This error is
/// produced by the [`validate_schema`] function, which also dumps
/// more error details to the console.
#[derive(Debug, Clone)]
pub struct UserFacingSchemaError {
  _priv: (),
}

/// Private implementation of some of the specific errors used inside
/// [`ArgumentSchemaError`] by this module.
#[derive(Error, Debug, Clone)]
enum ArgumentSchemaErrorImpl {
  #[error("expected {expected} argument(s), got {actual}")]
  WrongArity { expected: usize, actual: usize },
  #[error("expected {expected}, got \"{actual}\"")]
  TypeError { expected: String, actual: String },
}

impl NullaryArgumentSchema {
  pub fn new() -> Self {
    Self {
      _priv: (),
    }
  }
}

impl<P, T> UnaryArgumentSchema<P, T>
where P: Prism<String, T> {
  pub fn new(expected: String, prism: P) -> Self {
    Self {
      expected,
      prism,
      _phantom: PhantomData,
    }
  }
}

impl UnaryArgumentSchema<Identity, String> {
  /// Helper function for a unary argument schema which accepts any
  /// one argument.
  pub fn any() -> Self {
    Self::new("any".to_owned(), Identity)
  }
}

impl<P1, T1, P2, T2> BinaryArgumentSchema<P1, T1, P2, T2>
where P1: Prism<String, T1>,
      P2: Prism<String, T2> {
  pub fn new(first_expected: String, first_prism: P1, second_expected: String, second_prism: P2) -> Self {
    Self {
      first_expected,
      first_prism,
      second_expected,
      second_prism,
      _phantom: PhantomData,
    }
  }
}

impl ArgumentSchemaError {
  pub fn from_error(body: impl StdError + Send + Sync + 'static) -> Self {
    Self {
      body: Box::new(body),
    }
  }
}

impl ArgumentSchema for NullaryArgumentSchema {
  type Output = ();

  fn validate(&self, args: Vec<String>) -> Result<(), ArgumentSchemaError> {
    check_arity(&args, 0)
  }
}

impl<P, T> ArgumentSchema for UnaryArgumentSchema<P, T>
where P: Prism<String, T> {
  type Output = T;

  fn validate(&self, mut args: Vec<String>) -> Result<T, ArgumentSchemaError> {
    check_arity(&args, 1)?;
    let arg = args.pop().unwrap(); // unwrap: Just checked the arity.
    self.prism.narrow_type(arg).map_err(|arg| {
      ArgumentSchemaError::from_error(ArgumentSchemaErrorImpl::TypeError {
        expected: self.expected.clone(),
        actual: arg,
      })
    })
  }
}

impl<P1, T1, P2, T2> ArgumentSchema for BinaryArgumentSchema<P1, T1, P2, T2>
where P1: Prism<String, T1>,
      P2: Prism<String, T2> {
  type Output = (T1, T2);

  fn validate(&self, mut args: Vec<String>) -> Result<(T1, T2), ArgumentSchemaError> {
    check_arity(&args, 2)?;

    // unwrap: Just checked the arity.
    let arg2 = args.pop().unwrap();
    let arg1 = args.pop().unwrap();

    let arg1 = self.first_prism.narrow_type(arg1).map_err(|arg| {
      ArgumentSchemaError::from_error(ArgumentSchemaErrorImpl::TypeError {
        expected: self.first_expected.clone(),
        actual: arg,
      })
    })?;
    let arg2 = self.second_prism.narrow_type(arg2).map_err(|arg| {
      ArgumentSchemaError::from_error(ArgumentSchemaErrorImpl::TypeError {
        expected: self.second_expected.clone(),
        actual: arg,
      })
    })?;
    Ok((arg1, arg2))
  }
}

impl Display for ArgumentSchemaError {
  fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), fmt::Error> {
    write!(f, "{}", self.body)
  }
}

impl StdError for ArgumentSchemaError {
  fn source(&self) -> Option<&(dyn StdError + 'static)> {
    Some(&*self.body)
  }
}

impl Display for UserFacingSchemaError {
  fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), fmt::Error> {
    write!(f, "An error occurred in the Tauri command schema; please report this as a bug!")
  }
}

impl StdError for UserFacingSchemaError {}

/// If the length of `args` is equal to `expected`, returns `Ok`.
/// Otherwise, returns an appropriate error.
pub fn check_arity<T>(args: &[T], expected: usize) -> Result<(), ArgumentSchemaError> {
  let actual = args.len();
  if actual != expected {
    Err(ArgumentSchemaError::from_error(ArgumentSchemaErrorImpl::WrongArity {
      expected,
      actual,
    }))
  } else {
    Ok(())
  }
}

/// Validates the schema. In case of schema failure, this function
/// prints out the error to stderr and returns a generic error,
/// suitable for displaying to the user.
///
/// If you want the actual error object directly, rather than a
/// generic user-facing error, use [`ArgumentSchema::validate`]
/// directly.
pub fn validate_schema<S: ArgumentSchema>(
  schema: &S,
  args: Vec<String>,
) -> Result<S::Output, UserFacingSchemaError> {
  match schema.validate(args) {
    Ok(output) => Ok(output),
    Err(e) => {
      eprintln!("Argument schema error: {}", e);
      Err(UserFacingSchemaError { _priv: () })
    }
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  /// Test prism that only accepts strings of length 1.
  struct StringToChar;

  /// Test prism that only accepts the empty string.
  struct StringToUnit;

  impl Prism<String, char> for StringToChar {
    fn narrow_type(&self, arg: String) -> Result<char, String> {
      if arg.len() == 1 {
        Ok(arg.chars().next().unwrap())
      } else {
        Err(arg)
      }
    }
    fn widen_type(&self, arg: char) -> String {
      arg.to_string()
    }
  }

  impl Prism<String, ()> for StringToUnit {
    fn narrow_type(&self, arg: String) -> Result<(), String> {
      if arg.is_empty() {
        Ok(())
      } else {
        Err(arg)
      }
    }
    fn widen_type(&self, _: ()) -> String {
      "".to_string()
    }
  }

  #[test]
  fn test_check_arity_success() {
    check_arity::<()>(&[], 0).unwrap();
    check_arity(&[()], 1).unwrap();
    check_arity(&[(), ()], 2).unwrap();
    check_arity(&[(), (), (), (), (), ()], 6).unwrap();
  }

  #[test]
  fn test_check_arity_failure_zero() {
    let err = check_arity(&[()], 0).unwrap_err();
    assert_eq!(err.to_string(), "expected 0 argument(s), got 1");
  }

  #[test]
  fn test_check_arity_failure_two() {
    let err = check_arity(&[()], 2).unwrap_err();
    assert_eq!(err.to_string(), "expected 2 argument(s), got 1");
  }

  #[test]
  fn test_nullary_argument_schema_success() {
    let args = vec![];
    NullaryArgumentSchema::new().validate(args).unwrap();
  }

  #[test]
  fn test_nullary_argument_schema_failure() {
    let args = vec![String::from("foo")];
    let err = NullaryArgumentSchema::new().validate(args).unwrap_err();
    assert_eq!(err.to_string(), "expected 0 argument(s), got 1");
  }

  #[test]
  fn test_unary_argument_schema_identity_prism() {
    let schema = UnaryArgumentSchema::new("any".to_owned(), Identity);

    // Successes
    schema.validate(vec![String::from("xyz")]).unwrap();
    schema.validate(vec![String::from("0")]).unwrap();
    schema.validate(vec![String::from("")]).unwrap();

    // Failures
    let err = schema.validate(vec![]).unwrap_err();
    assert_eq!(err.to_string(), "expected 1 argument(s), got 0");
    let err = schema.validate(vec![String::from("a"), String::from("b")]).unwrap_err();
    assert_eq!(err.to_string(), "expected 1 argument(s), got 2");
  }

  #[test]
  fn test_unary_argument_schema_specific_prism() {
    let schema = UnaryArgumentSchema::new("one character".to_owned(), StringToChar);

    // Successes
    assert_eq!(schema.validate(vec![String::from("a")]).unwrap(), 'a');
    assert_eq!(schema.validate(vec![String::from("0")]).unwrap(), '0');
    assert_eq!(schema.validate(vec![String::from("_")]).unwrap(), '_');

    // Failures (by length)
    let err = schema.validate(vec![]).unwrap_err();
    assert_eq!(err.to_string(), "expected 1 argument(s), got 0");
    let err = schema.validate(vec![String::from("a"), String::from("b")]).unwrap_err();
    assert_eq!(err.to_string(), "expected 1 argument(s), got 2");
    let err = schema.validate(vec![String::from("xyzXYZ"), String::from("100")]).unwrap_err();
    assert_eq!(err.to_string(), "expected 1 argument(s), got 2");

    // Failures (by type)
    let err = schema.validate(vec![String::from("")]).unwrap_err();
    assert_eq!(err.to_string(), "expected one character, got \"\"");
    let err = schema.validate(vec![String::from("foo")]).unwrap_err();
    assert_eq!(err.to_string(), "expected one character, got \"foo\"");
  }

  #[test]
  fn test_binary_argument_schema_identity_prism() {
    let schema = BinaryArgumentSchema::new("any".to_owned(), Identity, "any".to_owned(), Identity);

    // Successes
    schema.validate(vec![String::from("xyz"), String::from("9")]).unwrap();
    schema.validate(vec![String::from("0"), String::from("0")]).unwrap();
    schema.validate(vec![String::from(""), String::from("potato")]).unwrap();

    // Failures
    let err = schema.validate(vec![]).unwrap_err();
    assert_eq!(err.to_string(), "expected 2 argument(s), got 0");
    let err = schema.validate(vec![String::from("foobarbaz")]).unwrap_err();
    assert_eq!(err.to_string(), "expected 2 argument(s), got 1");
    let err = schema.validate(vec![String::from("a"), String::from("b"), String::from("c")]).unwrap_err();
    assert_eq!(err.to_string(), "expected 2 argument(s), got 3");
  }

  #[test]
  fn test_binary_argument_schema_specific_prisms() {
    let schema = BinaryArgumentSchema::new(
      "one character".to_owned(),
      StringToChar,
      "empty string".to_owned(),
      StringToUnit,
    );

    // Successes
    assert_eq!(schema.validate(vec![String::from("a"), String::from("")]).unwrap(), ('a', ()));
    assert_eq!(schema.validate(vec![String::from("0"), String::from("")]).unwrap(), ('0', ()));

    // Failures (by length)
    let err = schema.validate(vec![]).unwrap_err();
    assert_eq!(err.to_string(), "expected 2 argument(s), got 0");
    let err = schema.validate(vec![String::from("xyzXYZ"), String::from("100"), String::from("9")]).unwrap_err();
    assert_eq!(err.to_string(), "expected 2 argument(s), got 3");

    // Failures (by first type)
    let err = schema.validate(vec![String::from(""), String::from("")]).unwrap_err();
    assert_eq!(err.to_string(), "expected one character, got \"\"");
    let err = schema.validate(vec![String::from(""), String::from("abcd")]).unwrap_err();
    assert_eq!(err.to_string(), "expected one character, got \"\"");

    // Failures (by second type)
    let err = schema.validate(vec![String::from("e"), String::from("0")]).unwrap_err();
    assert_eq!(err.to_string(), "expected empty string, got \"0\"");
  }
}
