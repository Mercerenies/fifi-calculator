
//! Builder API for [`Function`](super::function::Function) objects.

use super::function::Function;
use super::typechecker::{TypeChecker, Identity};
use crate::expr::Expr;
use crate::expr::simplifier::error::SimplifierError;
use crate::errorlist::ErrorList;

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

pub struct OneArgumentMatcher<C> {
  type_checker: C,
}

pub struct TwoArgumentMatcher<C1, C2> {
  first_type_checker: C1,
  second_type_checker: C2,
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

impl<C: TypeChecker<Expr>> OneArgumentMatcher<C> {
  pub fn of_type<D: TypeChecker<Expr>>(self, type_checker: D) -> OneArgumentMatcher<D> {
    OneArgumentMatcher { type_checker }
  }

  pub fn and_then<F>(self, f: F) -> Box<FunctionCase>
  where F: Fn(C::Output, &mut ErrorList<SimplifierError>) -> Result<Expr, Vec<Expr>> + Send + Sync + 'static,
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

impl<C1: TypeChecker<Expr>, C2: TypeChecker<Expr>> TwoArgumentMatcher<C1, C2> {
  pub fn of_types<D1: TypeChecker<Expr>, D2: TypeChecker<Expr>>(
    self,
    first_type_checker: D1,
    second_type_checker: D2,
  ) -> TwoArgumentMatcher<D1, D2> {
    TwoArgumentMatcher { first_type_checker, second_type_checker }
  }

  pub fn and_then<F>(self, f: F) -> Box<FunctionCase>
  where F: Fn(C1::Output, C2::Output, &mut ErrorList<SimplifierError>) -> Result<Expr, Vec<Expr>> + Send + Sync + 'static,
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

pub fn arity_one() -> OneArgumentMatcher<Identity<Expr>> {
  OneArgumentMatcher {
    type_checker: Identity::new(),
  }
}

pub fn arity_two() -> TwoArgumentMatcher<Identity<Expr>, Identity<Expr>> {
  TwoArgumentMatcher {
    first_type_checker: Identity::new(),
    second_type_checker: Identity::new(),
  }
}
