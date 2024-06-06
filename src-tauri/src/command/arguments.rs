
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

use crate::util::prism::Prism;

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
