
//! Builder API for [`Function`](super::function::Function) objects.

use super::function::Function;
use crate::util::prism::{Prism, Identity, OnVec};
use crate::expr::Expr;
use crate::expr::simplifier::error::SimplifierError;
use crate::errorlist::ErrorList;

use std::marker::PhantomData;

pub struct FunctionBuilder {
  name: String,
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
pub struct OneArgumentMatcher<C, Down> {
  arg_prism: C,
  _phantom: PhantomData<Down>,
}

/// Matcher that requires exactly two arguments in order to match. The
/// arguments can each be narrowed by prisms.
pub struct TwoArgumentMatcher<C1, C2, Down1, Down2> {
  first_arg_prism: C1,
  second_arg_prism: C2,
  _phantom: PhantomData<(Down1, Down2)>,
}

/// Matcher that accepts a variable number of arguments, possibly with
/// some arbitrary interval restriction on the number of arguments.
/// The arguments can uniformly be narrowed by a single prism.
pub struct VecMatcher<C, Down> {
  arg_prism: C,
  min_length: usize,
  max_length: usize,
  _phantom: PhantomData<Down>,
}

impl FunctionBuilder {
  pub fn new(name: impl Into<String>) -> Self {
    Self {
      name: name.into(),
      cases: Vec::new(),
    }
  }

  pub fn add_case(mut self, case: Box<FunctionCase>) -> Self {
    self.cases.push(case);
    self
  }

  pub fn build(self) -> Function {
    Function::new(
      self.name,
      Box::new(move |mut args, errors: &mut ErrorList<SimplifierError>| {
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
      }),
    )
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

impl<Down, C: Prism<Expr, Down>> OneArgumentMatcher<C, Down> {
  pub fn of_type<NewDown, D>(self, arg_prism: D) -> OneArgumentMatcher<D, NewDown>
  where D: Prism<Expr, NewDown> {
    OneArgumentMatcher { arg_prism, _phantom: PhantomData }
  }

  pub fn and_then<F>(self, f: F) -> Box<FunctionCase>
  where F: Fn(Down, &mut ErrorList<SimplifierError>) -> Result<Expr, Vec<Expr>> + Send + Sync + 'static,
        C: Send + Sync + 'static {
    Box::new(move |mut args, errors| {
      if args.len() != 1 {
        return FunctionCaseResult::NoMatch(args);
      }
      let arg = args.pop().unwrap(); // unwrap: args.len() == 1
      match self.arg_prism.narrow_type(arg) {
        Err(original_arg) => FunctionCaseResult::NoMatch(vec![original_arg]),
        Ok(arg) => FunctionCaseResult::from_result(f(arg, errors)),
      }
    })
  }
}

impl<Down1, Down2, C1, C2> TwoArgumentMatcher<C1, C2, Down1, Down2>
where C1: Prism<Expr, Down1>,
      C2: Prism<Expr, Down2> {
  pub fn of_types<NewDown1, NewDown2, D1, D2>(
    self,
    first_arg_prism: D1,
    second_arg_prism: D2,
  ) -> TwoArgumentMatcher<D1, D2, NewDown1, NewDown2>
  where D1: Prism<Expr, NewDown1>,
        D2: Prism<Expr, NewDown2> {
    TwoArgumentMatcher { first_arg_prism, second_arg_prism, _phantom: PhantomData }
  }

  pub fn and_then<F>(self, f: F) -> Box<FunctionCase>
  where F: Fn(Down1, Down2, &mut ErrorList<SimplifierError>) -> Result<Expr, Vec<Expr>> + Send + Sync + 'static,
        C1: Send + Sync + 'static,
        C2: Send + Sync + 'static {
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
            Ok(arg2) => FunctionCaseResult::from_result(f(arg1, arg2, errors)),
          }
        }
      }
    })
  }
}

impl<Down, C: Prism<Expr, Down>> VecMatcher<C, Down> {
  pub fn of_type<NewDown, D>(self, arg_prism: D) -> VecMatcher<D, NewDown>
  where D: Prism<Expr, NewDown> {
    VecMatcher {
      arg_prism,
      min_length: self.min_length,
      max_length: self.max_length,
      _phantom: PhantomData,
    }
  }

  pub fn and_then<F>(self, f: F) -> Box<FunctionCase>
  where F: Fn(Vec<Down>, &mut ErrorList<SimplifierError>) -> Result<Expr, Vec<Expr>> + Send + Sync + 'static,
        C: Send + Sync + 'static {
    let arg_prism = OnVec::new(self.arg_prism);
    Box::new(move |args, errors| {
      // Check arity
      if args.len() < self.min_length || args.len() > self.max_length {
        return FunctionCaseResult::NoMatch(args);
      }

      match arg_prism.narrow_type(args) {
        Err(args) => FunctionCaseResult::NoMatch(args),
        Ok(args) => FunctionCaseResult::from_result(f(args, errors)),
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
