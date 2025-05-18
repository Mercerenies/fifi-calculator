
use super::{FunctionCase, FunctionCaseResult};
use crate::util::prism::{Prism, Identity, OnVec};
use crate::expr::Expr;
use crate::expr::function::FunctionContext;
use crate::expr::simplifier::error::ArityError;
use crate::expr::calculus::{DerivativeEngine, DifferentiationFailure, DifferentiationError};
use crate::util::tuple::binder::narrow_vec;

/// Matcher that requires exactly one argument in order to match. The
/// argument can optionally be narrowed by a prism, which defaults to
/// [`Identity`].
///
/// This value is usually constructed with [`arity_one`].
pub struct OneArgumentMatcher<P, Down> {
  arg_prism: P,
  filter_fn: Option<Box<dyn Fn(&Down) -> bool + Send + Sync + 'static>>,
}

/// Matcher that requires exactly two arguments in order to match. The
/// arguments can each be narrowed by prisms.
///
/// This value is usually constructed with [`arity_two`].
pub struct TwoArgumentMatcher<P1, P2, Down1, Down2> {
  first_arg_prism: P1,
  second_arg_prism: P2,
  filter_fn: Option<Box<dyn Fn(&Down1, &Down2) -> bool + Send + Sync + 'static>>,
}

/// Matcher that requires exactly three arguments in order to match.
/// The arguments can each be narrowed by prisms.
///
/// This value is usually constructed with [`arity_three`].
pub struct ThreeArgumentMatcher<P1, P2, P3, Down1, Down2, Down3> {
  first_arg_prism: P1,
  second_arg_prism: P2,
  third_arg_prism: P3,
  filter_fn: Option<Box<dyn Fn(&Down1, &Down2, &Down3) -> bool + Send + Sync + 'static>>,
}

/// Matcher that requires exactly four arguments in order to match.
/// The arguments can each be narrowed by prisms.
///
/// This value is usually constructed with [`arity_four`].
pub struct FourArgumentMatcher<P1, P2, P3, P4, Down1, Down2, Down3, Down4> {
  first_arg_prism: P1,
  second_arg_prism: P2,
  third_arg_prism: P3,
  fourth_arg_prism: P4,
  filter_fn: Option<Box<dyn Fn(&Down1, &Down2, &Down3, &Down4) -> bool + Send + Sync + 'static>>,
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
  filter_fn: Option<Box<dyn Fn(&[Down]) -> bool + Send + Sync + 'static>>,
}

impl<Down: 'static, P: Prism<Expr, Down>> OneArgumentMatcher<P, Down> {
  /// Sets the prism for this matcher and resets the filter function.
  pub fn of_type<NewDown, Q>(self, arg_prism: Q) -> OneArgumentMatcher<Q, NewDown>
  where Q: Prism<Expr, NewDown> {
    OneArgumentMatcher { arg_prism, filter_fn: None }
  }

  pub fn filter(self, pred: impl Fn(&Down) -> bool + Send + Sync + 'static) -> OneArgumentMatcher<P, Down> {
    OneArgumentMatcher { arg_prism: self.arg_prism, filter_fn: Some(Box::new(pred)) }
  }

  pub fn and_then<T, F>(self, f: F) -> Box<FunctionCase<T>>
  where F: Fn(Down, &mut FunctionContext) -> Result<T, Down> + Send + Sync + 'static,
        P: Send + Sync + 'static {
    Box::new(move |args, context| {
      match narrow_vec((&self.arg_prism,), args) {
        Err(original_args) => FunctionCaseResult::NoMatch(original_args),
        Ok((arg,)) => {
          if let Some(filter_fn) = &self.filter_fn {
            if !filter_fn(&arg) {
              return FunctionCaseResult::NoMatch(vec![self.arg_prism.widen_type(arg)]);
            }
          }
          FunctionCaseResult::from_result(
            f(arg, context).map_err(|arg| vec![
              self.arg_prism.widen_type(arg),
            ]),
          )
        }
      }
    })
  }
}

impl<Down1: 'static, Down2: 'static, P1, P2> TwoArgumentMatcher<P1, P2, Down1, Down2>
where P1: Prism<Expr, Down1>,
      P2: Prism<Expr, Down2> {
  /// Sets the prisms for this matcher and resets the filter function.
  pub fn of_types<NewDown1, NewDown2, Q1, Q2>(
    self,
    first_arg_prism: Q1,
    second_arg_prism: Q2,
  ) -> TwoArgumentMatcher<Q1, Q2, NewDown1, NewDown2>
  where Q1: Prism<Expr, NewDown1>,
        Q2: Prism<Expr, NewDown2> {
    TwoArgumentMatcher { first_arg_prism, second_arg_prism, filter_fn: None }
  }

  /// Sets all of the prisms for this matcher to be the same, and
  /// resets the filter function.
  pub fn both_of_type<NewDown, Q>(self, arg_prism: Q) -> TwoArgumentMatcher<Q, Q, NewDown, NewDown>
  where Q: Prism<Expr, NewDown> + Clone {
    TwoArgumentMatcher {
      first_arg_prism: arg_prism.clone(),
      second_arg_prism: arg_prism,
      filter_fn: None,
    }
  }

  pub fn filter(self, pred: impl Fn(&Down1, &Down2) -> bool + Send + Sync + 'static) -> TwoArgumentMatcher<P1, P2, Down1, Down2> {
    TwoArgumentMatcher {
      first_arg_prism: self.first_arg_prism,
      second_arg_prism: self.second_arg_prism,
      filter_fn: Some(Box::new(pred)),
    }
  }

  pub fn and_then<T, F>(self, f: F) -> Box<FunctionCase<T>>
  where F: Fn(Down1, Down2, &mut FunctionContext) -> Result<T, (Down1, Down2)> + Send + Sync + 'static,
        P1: Send + Sync + 'static,
        P2: Send + Sync + 'static {
    Box::new(move |args, context| {
      match narrow_vec((&self.first_arg_prism, &self.second_arg_prism), args) {
        Err(original_args) => FunctionCaseResult::NoMatch(original_args),
        Ok((arg1, arg2)) => {
          if let Some(filter_fn) = &self.filter_fn {
            if !filter_fn(&arg1, &arg2) {
              return FunctionCaseResult::NoMatch(vec![
                self.first_arg_prism.widen_type(arg1),
                self.second_arg_prism.widen_type(arg2),
              ]);
            }
          }
          FunctionCaseResult::from_result(
            f(arg1, arg2, context).map_err(|(arg1, arg2)| {
              vec![
                self.first_arg_prism.widen_type(arg1),
                self.second_arg_prism.widen_type(arg2),
              ]
            }),
          )
        }
      }
    })
  }
}

impl<Down1: 'static, Down2: 'static, Down3: 'static, P1, P2, P3> ThreeArgumentMatcher<P1, P2, P3, Down1, Down2, Down3>
where P1: Prism<Expr, Down1>,
      P2: Prism<Expr, Down2>,
      P3: Prism<Expr, Down3> {
  /// Sets the prisms for this matcher and resets the filter function.
  pub fn of_types<NewDown1, NewDown2, NewDown3, Q1, Q2, Q3>(
    self,
    first_arg_prism: Q1,
    second_arg_prism: Q2,
    third_arg_prism: Q3,
  ) -> ThreeArgumentMatcher<Q1, Q2, Q3, NewDown1, NewDown2, NewDown3>
  where Q1: Prism<Expr, NewDown1>,
        Q2: Prism<Expr, NewDown2>,
        Q3: Prism<Expr, NewDown3> {
    ThreeArgumentMatcher { first_arg_prism, second_arg_prism, third_arg_prism, filter_fn: None }
  }

  /// Sets all of the prisms for this matcher to be the same, and
  /// resets the filter function.
  pub fn all_of_type<NewDown, Q>(self, arg_prism: Q) -> ThreeArgumentMatcher<Q, Q, Q, NewDown, NewDown, NewDown>
  where Q: Prism<Expr, NewDown> + Clone {
    ThreeArgumentMatcher {
      first_arg_prism: arg_prism.clone(),
      second_arg_prism: arg_prism.clone(),
      third_arg_prism: arg_prism,
      filter_fn: None,
    }
  }

  pub fn filter(self, pred: impl Fn(&Down1, &Down2, &Down3) -> bool + Send + Sync + 'static) -> ThreeArgumentMatcher<P1, P2, P3, Down1, Down2, Down3> {
    ThreeArgumentMatcher {
      first_arg_prism: self.first_arg_prism,
      second_arg_prism: self.second_arg_prism,
      third_arg_prism: self.third_arg_prism,
      filter_fn: Some(Box::new(pred)),
    }
  }

  pub fn and_then<T, F>(self, f: F) -> Box<FunctionCase<T>>
  where F: Fn(Down1, Down2, Down3, &mut FunctionContext) -> Result<T, (Down1, Down2, Down3)> + Send + Sync + 'static,
        P1: Send + Sync + 'static,
        P2: Send + Sync + 'static,
        P3: Send + Sync + 'static {
    Box::new(move |args, context| {
      match narrow_vec((&self.first_arg_prism, &self.second_arg_prism, &self.third_arg_prism), args) {
        Err(original_args) => FunctionCaseResult::NoMatch(original_args),
        Ok((arg1, arg2, arg3)) => {
          if let Some(filter_fn) = &self.filter_fn {
            if !filter_fn(&arg1, &arg2, &arg3) {
              return FunctionCaseResult::NoMatch(vec![
                self.first_arg_prism.widen_type(arg1),
                self.second_arg_prism.widen_type(arg2),
                self.third_arg_prism.widen_type(arg3),
              ]);
            }
          }
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
    })
  }
}

impl<Down1: 'static, Down2: 'static, Down3: 'static, Down4: 'static, P1, P2, P3, P4> FourArgumentMatcher<P1, P2, P3, P4, Down1, Down2, Down3, Down4>
where P1: Prism<Expr, Down1>,
      P2: Prism<Expr, Down2>,
      P3: Prism<Expr, Down3>,
      P4: Prism<Expr, Down4> {
  /// Sets the prisms for this matcher and resets the filter function.
  pub fn of_types<NewDown1, NewDown2, NewDown3, NewDown4, Q1, Q2, Q3, Q4>(
    self,
    first_arg_prism: Q1,
    second_arg_prism: Q2,
    third_arg_prism: Q3,
    fourth_arg_prism: Q4,
  ) -> FourArgumentMatcher<Q1, Q2, Q3, Q4, NewDown1, NewDown2, NewDown3, NewDown4>
  where Q1: Prism<Expr, NewDown1>,
        Q2: Prism<Expr, NewDown2>,
        Q3: Prism<Expr, NewDown3>,
        Q4: Prism<Expr, NewDown4> {
    FourArgumentMatcher {
      first_arg_prism,
      second_arg_prism,
      third_arg_prism,
      fourth_arg_prism,
      filter_fn: None,
    }
  }

  /// Sets all of the prisms for this matcher to be the same, and
  /// resets the filter function.
  pub fn all_of_type<NewDown, Q>(
    self,
    arg_prism: Q,
  ) -> FourArgumentMatcher<Q, Q, Q, Q, NewDown, NewDown, NewDown, NewDown>
  where Q: Prism<Expr, NewDown> + Clone {
    FourArgumentMatcher {
      first_arg_prism: arg_prism.clone(),
      second_arg_prism: arg_prism.clone(),
      third_arg_prism: arg_prism.clone(),
      fourth_arg_prism: arg_prism,
      filter_fn: None,
    }
  }

  pub fn filter(self, pred: impl Fn(&Down1, &Down2, &Down3, &Down4) -> bool + Send + Sync + 'static) -> FourArgumentMatcher<P1, P2, P3, P4, Down1, Down2, Down3, Down4> {
    FourArgumentMatcher {
      first_arg_prism: self.first_arg_prism,
      second_arg_prism: self.second_arg_prism,
      third_arg_prism: self.third_arg_prism,
      fourth_arg_prism: self.fourth_arg_prism,
      filter_fn: Some(Box::new(pred)),
    }
  }

  pub fn and_then<T, F>(self, f: F) -> Box<FunctionCase<T>>
  where F: Fn(Down1, Down2, Down3, Down4, &mut FunctionContext) -> Result<T, (Down1, Down2, Down3, Down4)> + Send + Sync + 'static,
        P1: Send + Sync + 'static,
        P2: Send + Sync + 'static,
        P3: Send + Sync + 'static,
        P4: Send + Sync + 'static {
    Box::new(move |args, context| {
      match narrow_vec((&self.first_arg_prism, &self.second_arg_prism, &self.third_arg_prism, &self.fourth_arg_prism), args) {
        Err(original_args) => FunctionCaseResult::NoMatch(original_args),
        Ok((arg1, arg2, arg3, arg4)) => {
          if let Some(filter_fn) = &self.filter_fn {
            if !filter_fn(&arg1, &arg2, &arg3, &arg4) {
              return FunctionCaseResult::NoMatch(vec![
                self.first_arg_prism.widen_type(arg1),
                self.second_arg_prism.widen_type(arg2),
                self.third_arg_prism.widen_type(arg3),
                self.fourth_arg_prism.widen_type(arg4),
              ]);
            }
          }
          FunctionCaseResult::from_result(
            f(arg1, arg2, arg3, arg4, context).map_err(|(arg1, arg2, arg3, arg4)| {
              vec![
                self.first_arg_prism.widen_type(arg1),
                self.second_arg_prism.widen_type(arg2),
                self.third_arg_prism.widen_type(arg3),
                self.fourth_arg_prism.widen_type(arg4),
              ]
            })
          )
        }
      }
    })
  }
}

impl<Down: 'static, P: Prism<Expr, Down>> VecMatcher<P, Down> {
  /// Sets the prism for this matcher and resets the filter function.
  pub fn of_type<NewDown, Q>(self, arg_prism: Q) -> VecMatcher<Q, NewDown>
  where Q: Prism<Expr, NewDown> {
    VecMatcher {
      arg_prism,
      min_length: self.min_length,
      max_length: self.max_length,
      filter_fn: None,
    }
  }

  pub fn filter(self, pred: impl Fn(&[Down]) -> bool + Send + Sync + 'static) -> VecMatcher<P, Down> {
    VecMatcher {
      arg_prism: self.arg_prism,
      min_length: self.min_length,
      max_length: self.max_length,
      filter_fn: Some(Box::new(pred)),
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
        Ok(args) => {
          if let Some(filter_fn) = &self.filter_fn {
            if !filter_fn(&args) {
              return FunctionCaseResult::NoMatch(
                arg_prism.widen_type(args),
              );
            }
          }
          FunctionCaseResult::from_result(
            f(args, context).map_err(|args| {
              arg_prism.widen_type(args)
            }),
          )
        }
      }
    })
  }
}

pub fn arity_one() -> OneArgumentMatcher<Identity, Expr> {
  OneArgumentMatcher {
    arg_prism: Identity,
    filter_fn: None,
  }
}

pub fn arity_two() -> TwoArgumentMatcher<Identity, Identity, Expr, Expr> {
  TwoArgumentMatcher {
    first_arg_prism: Identity,
    second_arg_prism: Identity,
    filter_fn: None,
  }
}

pub fn arity_three() -> ThreeArgumentMatcher<Identity, Identity, Identity, Expr, Expr, Expr> {
  ThreeArgumentMatcher {
    first_arg_prism: Identity,
    second_arg_prism: Identity,
    third_arg_prism: Identity,
    filter_fn: None,
  }
}

pub fn arity_four() -> FourArgumentMatcher<Identity, Identity, Identity, Identity, Expr, Expr, Expr, Expr> {
  FourArgumentMatcher {
    first_arg_prism: Identity,
    second_arg_prism: Identity,
    third_arg_prism: Identity,
    fourth_arg_prism: Identity,
    filter_fn: None,
  }
}

pub fn any_arity() -> VecMatcher<Identity, Expr> {
  VecMatcher {
    arg_prism: Identity,
    min_length: 0,
    max_length: usize::MAX,
    filter_fn: None,
  }
}

pub fn non_zero_arity() -> VecMatcher<Identity, Expr> {
  VecMatcher {
    arg_prism: Identity,
    min_length: 1,
    max_length: usize::MAX,
    filter_fn: None,
  }
}

pub fn exact_arity(arity: usize) -> VecMatcher<Identity, Expr> {
  VecMatcher {
    arg_prism: Identity,
    min_length: arity,
    max_length: arity,
    filter_fn: None,
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

