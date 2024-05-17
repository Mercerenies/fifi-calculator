
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

pub struct OneArgumentMatcher<C, Down> {
  type_checker: C,
  _phantom: PhantomData<Down>,
}

pub struct TwoArgumentMatcher<C1, C2, Down1, Down2> {
  first_type_checker: C1,
  second_type_checker: C2,
  _phantom: PhantomData<(Down1, Down2)>,
}

pub struct AnyArityMatcher<C, Down> {
  type_checker: C,
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
  pub fn of_type<NewDown, D>(self, type_checker: D) -> OneArgumentMatcher<D, NewDown>
  where D: Prism<Expr, NewDown> {
    OneArgumentMatcher { type_checker, _phantom: PhantomData }
  }

  pub fn and_then<F>(self, f: F) -> Box<FunctionCase>
  where F: Fn(Down, &mut ErrorList<SimplifierError>) -> Result<Expr, Vec<Expr>> + Send + Sync + 'static,
        C: Send + Sync + 'static {
    Box::new(move |mut args, errors| {
      if args.len() != 1 {
        return FunctionCaseResult::NoMatch(args);
      }
      let arg = args.pop().unwrap(); // unwrap: args.len() == 1
      match self.type_checker.narrow_type(arg) {
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
    first_type_checker: D1,
    second_type_checker: D2,
  ) -> TwoArgumentMatcher<D1, D2, NewDown1, NewDown2>
  where D1: Prism<Expr, NewDown1>,
        D2: Prism<Expr, NewDown2> {
    TwoArgumentMatcher { first_type_checker, second_type_checker, _phantom: PhantomData }
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
      match self.first_type_checker.narrow_type(arg1) {
        Err(original_arg1) => FunctionCaseResult::NoMatch(vec![original_arg1, arg2]),
        Ok(arg1) => {
          match self.second_type_checker.narrow_type(arg2) {
            Err(original_arg2) => {
              let original_arg1 = self.first_type_checker.widen_type(arg1);
              FunctionCaseResult::NoMatch(vec![original_arg1, original_arg2])
            }
            Ok(arg2) => FunctionCaseResult::from_result(f(arg1, arg2, errors)),
          }
        }
      }
    })
  }
}

impl<Down, C: Prism<Expr, Down>> AnyArityMatcher<C, Down> {
  pub fn of_type<NewDown, D>(self, type_checker: D) -> AnyArityMatcher<D, NewDown>
  where D: Prism<Expr, NewDown> {
    AnyArityMatcher { type_checker, _phantom: PhantomData }
  }

  pub fn and_then<F>(self, f: F) -> Box<FunctionCase>
  where F: Fn(Vec<Down>, &mut ErrorList<SimplifierError>) -> Result<Expr, Vec<Expr>> + Send + Sync + 'static,
        C: Send + Sync + 'static {
    let type_checker = OnVec::new(self.type_checker);
    Box::new(move |args, errors| {
      match type_checker.narrow_type(args) {
        Err(args) => FunctionCaseResult::NoMatch(args),
        Ok(args) => FunctionCaseResult::from_result(f(args, errors)),
      }
    })
  }
}

pub fn arity_one() -> OneArgumentMatcher<Identity, Expr> {
  OneArgumentMatcher {
    type_checker: Identity::new(),
    _phantom: PhantomData,
  }
}

pub fn arity_two() -> TwoArgumentMatcher<Identity, Identity, Expr, Expr> {
  TwoArgumentMatcher {
    first_type_checker: Identity::new(),
    second_type_checker: Identity::new(),
    _phantom: PhantomData,
  }
}
