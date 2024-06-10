
//! Builder API for [`Function`] objects.

use super::Function;
use super::flags::FunctionFlags;
use crate::util::prism::{Prism, Identity, OnVec};
use crate::expr::Expr;
use crate::expr::simplifier::error::SimplifierError;
use crate::errorlist::ErrorList;

use std::marker::PhantomData;

pub struct FunctionBuilder {
  name: String,
  flags: FunctionFlags,
  identity_predicate: Box<dyn Fn(&Expr) -> bool + Send + Sync + 'static>,
  cases: Vec<Box<FunctionCase>>,
}

pub type FunctionCase =
  dyn Fn(Vec<Expr>, &mut ErrorList<SimplifierError>) -> FunctionCaseResult + Send + Sync;

/// Result of attempting to apply a function match case.
pub enum FunctionCaseResult {
  /// Indicates that the function evaluation succeeded, with the given
  /// result value.
  Success(Expr),
  /// Indicates that the function case matched but evaluation failed,
  /// and returns ownership of the original arguments to the caller.
  ///
  /// This aborts pattern matching and does NOT continue with
  /// additional branches. Usually, errors will be reported in the
  /// `ErrorList` when this value is returned.
  Failure(Vec<Expr>),
  /// Indicates that the function case did not match and that pattern
  /// matching should continue with the next branch.
  NoMatch(Vec<Expr>),
}

/// Matcher that requires exactly one argument in order to match. The
/// argument can optionally be narrowed by a prism, which defaults to
/// [`Identity`].
///
/// This value is usually constructed with [`arity_one`].
pub struct OneArgumentMatcher<P, Down> {
  arg_prism: P,
  _phantom: PhantomData<fn() -> Down>,
}

/// Matcher that requires exactly two arguments in order to match. The
/// arguments can each be narrowed by prisms.
///
/// This value is usually constructed with [`arity_two`].
pub struct TwoArgumentMatcher<P1, P2, Down1, Down2> {
  first_arg_prism: P1,
  second_arg_prism: P2,
  _phantom: PhantomData<fn() -> (Down1, Down2)>,
}

/// Matcher that accepts a variable number of arguments, possibly with
/// some arbitrary interval restriction on the number of arguments.
/// The arguments can uniformly be narrowed by a single prism.
///
/// This value is usually constructed with [`any_arity`] or
/// [`non_zero_arity`].
pub struct VecMatcher<P, Down> {
  arg_prism: P,
  min_length: usize,
  max_length: usize,
  _phantom: PhantomData<fn() -> Down>,
}

impl FunctionBuilder {
  /// Constructs a new `FunctionBuilder` object for the function with
  /// the given name.
  pub fn new(name: impl Into<String>) -> Self {
    Self {
      name: name.into(),
      identity_predicate: Box::new(super::no_identity_value),
      flags: FunctionFlags::default(),
      cases: Vec::new(),
    }
  }

  /// Adds an evaluation case to `self`. This function is intended to
  /// be called in fluent style, and it returns `self` after
  /// modifications.
  pub fn add_case(mut self, case: Box<FunctionCase>) -> Self {
    self.cases.push(case);
    self
  }

  /// Sets the predicate for recognizing identity values for the
  /// function being built. Modifies and returns `self`, to permit
  /// fluent-style calling.
  pub fn set_identity(mut self, predicate: impl Fn(&Expr) -> bool + Send + Sync + 'static) -> Self {
    self.identity_predicate = Box::new(predicate);
    self
  }

  /// Enables the [`PERMITS_FLATTENING`](FunctionFlags::PERMITS_FLATTENING)
  /// flag for `self`.
  pub fn permit_flattening(mut self) -> Self {
    self.flags |= FunctionFlags::PERMITS_FLATTENING;
    self
  }

  /// Consumes `self` and builds it into a completed [`Function`]
  /// value.
  pub fn build(self) -> Function {
    let function_body = Box::new(move |mut args, errors: &mut ErrorList<SimplifierError>| {
      for case in &self.cases {
        match case(args, &mut *errors) {
          FunctionCaseResult::Success(output) => {
            return Ok(output);
          }
          FunctionCaseResult::Failure(args) => {
            return Err(args);
          }
          FunctionCaseResult::NoMatch(original_args) => {
            args = original_args;
          }
        }
      }
      // No cases matched, so we refuse to evaluate the function.
      Err(args)
    });
    Function {
      name: self.name,
      flags: self.flags,
      identity_predicate: self.identity_predicate,
      body: function_body,
    }
  }
}

impl FunctionCaseResult {
  /// Reports `Ok` as `Success` and `Err` as `Failure`. This function
  /// always reports a successful match, so `NoMatch` will never be
  /// returned.
  fn from_result(expr: Result<Expr, Vec<Expr>>) -> Self {
    match expr {
      Ok(expr) => FunctionCaseResult::Success(expr),
      Err(args) => FunctionCaseResult::Failure(args),
    }
  }
}

impl<Down, P: Prism<Expr, Down>> OneArgumentMatcher<P, Down> {
  pub fn of_type<NewDown, Q>(self, arg_prism: Q) -> OneArgumentMatcher<Q, NewDown>
  where Q: Prism<Expr, NewDown> {
    OneArgumentMatcher { arg_prism, _phantom: PhantomData }
  }

  pub fn and_then<F>(self, f: F) -> Box<FunctionCase>
  where F: Fn(Down, &mut ErrorList<SimplifierError>) -> Result<Expr, Down> + Send + Sync + 'static,
        P: Send + Sync + 'static {
    Box::new(move |mut args, errors| {
      if args.len() != 1 {
        return FunctionCaseResult::NoMatch(args);
      }
      let arg = args.pop().unwrap(); // unwrap: args.len() == 1
      match self.arg_prism.narrow_type(arg) {
        Err(original_arg) => FunctionCaseResult::NoMatch(vec![original_arg]),
        Ok(arg) => FunctionCaseResult::from_result(
          f(arg, errors).map_err(|arg| {
            vec![self.arg_prism.widen_type(arg)]
          }),
        ),
      }
    })
  }
}

impl<Down1, Down2, P1, P2> TwoArgumentMatcher<P1, P2, Down1, Down2>
where P1: Prism<Expr, Down1>,
      P2: Prism<Expr, Down2> {
  pub fn of_types<NewDown1, NewDown2, Q1, Q2>(
    self,
    first_arg_prism: Q1,
    second_arg_prism: Q2,
  ) -> TwoArgumentMatcher<Q1, Q2, NewDown1, NewDown2>
  where Q1: Prism<Expr, NewDown1>,
        Q2: Prism<Expr, NewDown2> {
    TwoArgumentMatcher { first_arg_prism, second_arg_prism, _phantom: PhantomData }
  }

  pub fn both_of_type<NewDown, Q>(self, arg_prism: Q) -> TwoArgumentMatcher<Q, Q, NewDown, NewDown>
  where Q: Prism<Expr, NewDown> + Clone {
   TwoArgumentMatcher {
      first_arg_prism: arg_prism.clone(),
      second_arg_prism: arg_prism,
      _phantom: PhantomData,
    }
  }

  pub fn and_then<F>(self, f: F) -> Box<FunctionCase>
  where F: Fn(Down1, Down2, &mut ErrorList<SimplifierError>) -> Result<Expr, (Down1, Down2)> + Send + Sync + 'static,
        P1: Send + Sync + 'static,
        P2: Send + Sync + 'static {
    Box::new(move |mut args, errors| {
      if args.len() != 2 {
        return FunctionCaseResult::NoMatch(args);
      }
      let arg2 = args.pop().unwrap(); // unwrap: args.len() == 2
      let arg1 = args.pop().unwrap(); // unwrap: args.len() == 2
      match self.first_arg_prism.narrow_type(arg1) {
        Err(original_arg1) => FunctionCaseResult::NoMatch(vec![original_arg1, arg2]),
        Ok(arg1) => {
          match self.second_arg_prism.narrow_type(arg2) {
            Err(original_arg2) => {
              let original_arg1 = self.first_arg_prism.widen_type(arg1);
              FunctionCaseResult::NoMatch(vec![original_arg1, original_arg2])
            }
            Ok(arg2) => {
              FunctionCaseResult::from_result(
                f(arg1, arg2, errors).map_err(|(arg1, arg2)| {
                  vec![self.first_arg_prism.widen_type(arg1), self.second_arg_prism.widen_type(arg2)]
                }),
              )
            }
          }
        }
      }
    })
  }
}

impl<Down, P: Prism<Expr, Down>> VecMatcher<P, Down> {
  pub fn of_type<NewDown, Q>(self, arg_prism: Q) -> VecMatcher<Q, NewDown>
  where Q: Prism<Expr, NewDown> {
    VecMatcher {
      arg_prism,
      min_length: self.min_length,
      max_length: self.max_length,
      _phantom: PhantomData,
    }
  }

  pub fn and_then<F>(self, f: F) -> Box<FunctionCase>
  where F: Fn(Vec<Down>, &mut ErrorList<SimplifierError>) -> Result<Expr, Vec<Down>> + Send + Sync + 'static,
        P: Send + Sync + 'static {
    let arg_prism = OnVec::new(self.arg_prism);
    Box::new(move |args, errors| {
      // Check arity
      if args.len() < self.min_length || args.len() > self.max_length {
        return FunctionCaseResult::NoMatch(args);
      }

      match arg_prism.narrow_type(args) {
        Err(args) => FunctionCaseResult::NoMatch(args),
        Ok(args) => FunctionCaseResult::from_result(
          f(args, errors).map_err(|args| {
            arg_prism.widen_type(args)
          }),
        ),
      }
    })
  }
}

pub fn arity_one() -> OneArgumentMatcher<Identity, Expr> {
  OneArgumentMatcher {
    arg_prism: Identity::new(),
    _phantom: PhantomData,
  }
}

pub fn arity_two() -> TwoArgumentMatcher<Identity, Identity, Expr, Expr> {
  TwoArgumentMatcher {
    first_arg_prism: Identity::new(),
    second_arg_prism: Identity::new(),
    _phantom: PhantomData,
  }
}

pub fn any_arity() -> VecMatcher<Identity, Expr> {
  VecMatcher {
    arg_prism: Identity::new(),
    min_length: 0,
    max_length: usize::MAX,
    _phantom: PhantomData,
  }
}

pub fn non_zero_arity() -> VecMatcher<Identity, Expr> {
  VecMatcher {
    arg_prism: Identity::new(),
    min_length: 1,
    max_length: usize::MAX,
    _phantom: PhantomData,
  }
}