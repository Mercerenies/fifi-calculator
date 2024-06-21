
//! Builder API for [`Function`] objects.

use super::{Function, FunctionContext, FunctionDeriv};
use super::flags::FunctionFlags;
use crate::util::prism::{Prism, Identity, OnVec};
use crate::graphics::response::GraphicsDirective;
use crate::expr::Expr;
use crate::expr::simplifier::error::ArityError;
use crate::expr::calculus::{DerivativeEngine, DifferentiationFailure, DifferentiationError};

use std::marker::PhantomData;

pub struct FunctionBuilder {
  name: String,
  flags: FunctionFlags,
  identity_predicate: Box<dyn Fn(&Expr) -> bool + Send + Sync + 'static>,
  derivative_rule: Option<Box<FunctionDeriv>>,
  cases: Vec<Box<FunctionCase<Expr>>>,
  graphics_cases: Vec<Box<FunctionCase<GraphicsDirective>>>,
}

pub type FunctionCase<T> =
  dyn Fn(Vec<Expr>, &mut FunctionContext) -> FunctionCaseResult<T> + Send + Sync;

/// Result of attempting to apply a function match case.
pub enum FunctionCaseResult<T> {
  /// Indicates that the function evaluation succeeded, with the given
  /// result value.
  Success(T),
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

/// Matcher that requires exactly three arguments in order to match.
/// The arguments can each be narrowed by prisms.
///
/// This value is usually constructed with [`arity_three`].
pub struct ThreeArgumentMatcher<P1, P2, P3, Down1, Down2, Down3> {
  first_arg_prism: P1,
  second_arg_prism: P2,
  third_arg_prism: P3,
  #[allow(clippy::type_complexity)] // It's just a PhantomData, to get rid of unused args.
  _phantom: PhantomData<fn() -> (Down1, Down2, Down3)>,
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
      flags: FunctionFlags::default(),
      identity_predicate: Box::new(super::no_identity_value),
      derivative_rule: None,
      cases: Vec::new(),
      graphics_cases: Vec::new(),
    }
  }

  /// Adds an evaluation case to `self`. This function is intended to
  /// be called in fluent style, and it returns `self` after
  /// modifications.
  pub fn add_case(mut self, case: Box<FunctionCase<Expr>>) -> Self {
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

  /// Sets the rule for how to differentiate this function. If a
  /// derivative rule has already been set, then `set_derivative`
  /// panics.
  pub fn set_derivative(mut self, rule: impl Fn(Vec<Expr>, &DerivativeEngine) -> Result<Expr, DifferentiationFailure> + Send + Sync + 'static) -> Self {
    assert!(
      self.derivative_rule.is_none(),
      "Cannot set derivative rule on function {} that already has one.",
      self.name,
    );
    self.derivative_rule = Some(Box::new(rule));
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
    Function {
      name: self.name,
      flags: self.flags,
      identity_predicate: self.identity_predicate,
      derivative_rule: self.derivative_rule,
      body: build_function_body(self.cases),
      graphics_body: build_function_body(self.graphics_cases),
    }
  }
}

fn build_function_body<T: 'static>(cases: Vec<Box<FunctionCase<T>>>) -> Box<super::FunctionImpl<T>> {
  if cases.is_empty() {
    // If there are no cases (for instance, `graphics_body` for a
    // function which is not used in graphics), return a much simpler
    // closure, for efficiency reasons.
    return Box::new(|args, _| Err(args));
  }

  Box::new(move |mut args, mut context: FunctionContext| {
    for case in &cases {
      match case(args, &mut context) {
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
  })
}

impl<T> FunctionCaseResult<T> {
  /// Reports `Ok` as `Success` and `Err` as `Failure`. This function
  /// always reports a successful match, so `NoMatch` will never be
  /// returned.
  fn from_result(expr: Result<T, Vec<Expr>>) -> Self {
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

  pub fn and_then<T, F>(self, f: F) -> Box<FunctionCase<T>>
  where F: Fn(Down, &mut FunctionContext) -> Result<T, Down> + Send + Sync + 'static,
        P: Send + Sync + 'static {
    Box::new(move |mut args, context| {
      if args.len() != 1 {
        return FunctionCaseResult::NoMatch(args);
      }
      let arg = args.pop().unwrap(); // unwrap: args.len() == 1
      match self.arg_prism.narrow_type(arg) {
        Err(original_arg) => FunctionCaseResult::NoMatch(vec![original_arg]),
        Ok(arg) => FunctionCaseResult::from_result(
          f(arg, context).map_err(|arg| {
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

  pub fn and_then<T, F>(self, f: F) -> Box<FunctionCase<T>>
  where F: Fn(Down1, Down2, &mut FunctionContext) -> Result<T, (Down1, Down2)> + Send + Sync + 'static,
        P1: Send + Sync + 'static,
        P2: Send + Sync + 'static {
    Box::new(move |mut args, context| {
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
                f(arg1, arg2, context).map_err(|(arg1, arg2)| {
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

impl<Down1, Down2, Down3, P1, P2, P3> ThreeArgumentMatcher<P1, P2, P3, Down1, Down2, Down3>
where P1: Prism<Expr, Down1>,
      P2: Prism<Expr, Down2>,
      P3: Prism<Expr, Down3> {
  pub fn of_types<NewDown1, NewDown2, NewDown3, Q1, Q2, Q3>(
    self,
    first_arg_prism: Q1,
    second_arg_prism: Q2,
    third_arg_prism: Q3,
  ) -> ThreeArgumentMatcher<Q1, Q2, Q3, NewDown1, NewDown2, NewDown3>
  where Q1: Prism<Expr, NewDown1>,
        Q2: Prism<Expr, NewDown2>,
        Q3: Prism<Expr, NewDown3> {
    ThreeArgumentMatcher { first_arg_prism, second_arg_prism, third_arg_prism, _phantom: PhantomData }
  }

  pub fn all_of_type<NewDown, Q>(self, arg_prism: Q) -> ThreeArgumentMatcher<Q, Q, Q, NewDown, NewDown, NewDown>
  where Q: Prism<Expr, NewDown> + Clone {
   ThreeArgumentMatcher {
      first_arg_prism: arg_prism.clone(),
      second_arg_prism: arg_prism.clone(),
      third_arg_prism: arg_prism,
      _phantom: PhantomData,
    }
  }

  pub fn and_then<T, F>(self, f: F) -> Box<FunctionCase<T>>
  where F: Fn(Down1, Down2, Down3, &mut FunctionContext) -> Result<T, (Down1, Down2, Down3)> + Send + Sync + 'static,
        P1: Send + Sync + 'static,
        P2: Send + Sync + 'static,
        P3: Send + Sync + 'static {
    // TODO: Better way to avoid this pyramid of doom :(
    Box::new(move |mut args, context| {
      if args.len() != 3 {
        return FunctionCaseResult::NoMatch(args);
      }
      let arg3 = args.pop().unwrap(); // unwrap: args.len() == 3
      let arg2 = args.pop().unwrap(); // unwrap: args.len() == 3
      let arg1 = args.pop().unwrap(); // unwrap: args.len() == 3
      match self.first_arg_prism.narrow_type(arg1) {
        Err(original_arg1) => FunctionCaseResult::NoMatch(vec![original_arg1, arg2, arg3]),
        Ok(arg1) => {
          match self.second_arg_prism.narrow_type(arg2) {
            Err(original_arg2) => {
              let original_arg1 = self.first_arg_prism.widen_type(arg1);
              FunctionCaseResult::NoMatch(vec![original_arg1, original_arg2])
            }
            Ok(arg2) => {
              match self.third_arg_prism.narrow_type(arg3) {
                Err(original_arg_3) => {
                  let original_arg1 = self.first_arg_prism.widen_type(arg1);
                  let original_arg2 = self.second_arg_prism.widen_type(arg2);
                  FunctionCaseResult::NoMatch(vec![original_arg1, original_arg2, original_arg_3])
                }
                Ok(arg3) => {
                  FunctionCaseResult::from_result(
                    f(arg1, arg2, arg3, context).map_err(|(arg1, arg2, arg3)| {
                      vec![
                        self.first_arg_prism.widen_type(arg1),
                        self.second_arg_prism.widen_type(arg2),
                        self.third_arg_prism.widen_type(arg3),
                      ]
                    }),
                  )
                }
              }
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

  pub fn and_then<T, F>(self, f: F) -> Box<FunctionCase<T>>
  where F: Fn(Vec<Down>, &mut FunctionContext) -> Result<T, Vec<Down>> + Send + Sync + 'static,
        P: Send + Sync + 'static {
    let arg_prism = OnVec::new(self.arg_prism);
    Box::new(move |args, context| {
      // Check arity
      if args.len() < self.min_length || args.len() > self.max_length {
        return FunctionCaseResult::NoMatch(args);
      }

      match arg_prism.narrow_type(args) {
        Err(args) => FunctionCaseResult::NoMatch(args),
        Ok(args) => FunctionCaseResult::from_result(
          f(args, context).map_err(|args| {
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

pub fn arity_three() -> ThreeArgumentMatcher<Identity, Identity, Identity, Expr, Expr, Expr> {
  ThreeArgumentMatcher {
    first_arg_prism: Identity::new(),
    second_arg_prism: Identity::new(),
    third_arg_prism: Identity::new(),
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

pub fn arity_one_deriv(
  function_name: &str,
  f: impl Fn(Expr, &DerivativeEngine) -> Result<Expr, DifferentiationFailure> + Send + Sync + 'static
) -> impl Fn(Vec<Expr>, &DerivativeEngine) -> Result<Expr, DifferentiationFailure> + Send + Sync + 'static {
  let function_name = function_name.to_owned();
  move |mut args, engine| {
    if args.len() != 1 {
      let err = ArityError { expected: 1, actual: args.len() };
      return Err(engine.error(DifferentiationError::ArityError(function_name.clone(), err)));
    }
    let arg = args.pop().unwrap(); // unwrap: len() == 1
    f(arg, engine)
  }
}

pub fn arity_two_deriv(
  function_name: &str,
  f: impl Fn(Expr, Expr, &DerivativeEngine) -> Result<Expr, DifferentiationFailure> + Send + Sync + 'static
) -> impl Fn(Vec<Expr>, &DerivativeEngine) -> Result<Expr, DifferentiationFailure> + Send + Sync + 'static {
  let function_name = function_name.to_owned();
  move |mut args, engine| {
    if args.len() != 2 {
      let err = ArityError { expected: 2, actual: args.len() };
      return Err(engine.error(DifferentiationError::ArityError(function_name.clone(), err)));
    }
    let arg2 = args.pop().unwrap(); // unwrap: len() == 2
    let arg1 = args.pop().unwrap(); // unwrap: len() == 2
    f(arg1, arg2, engine)
  }
}
