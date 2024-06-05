
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
  _priv: PhantomData<()>,
}

/// An error occurring during validation of an argument list schema.
#[derive(Debug)]
pub struct ArgumentSchemaError {
  body: Box<dyn StdError + Send + Sync + 'static>,
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
      _priv: PhantomData,
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
