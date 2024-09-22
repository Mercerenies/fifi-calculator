
//! Builder API for [`Function`] objects.

pub mod matcher;

pub use matcher::{arity_one, arity_two, arity_three, arity_four, any_arity,
                  arity_one_deriv, arity_two_deriv};

use super::{Function, FunctionContext, FunctionDeriv, FunctionImpl, no_identity_value};
use super::flags::FunctionFlags;
use super::partial::{simplify_sequences, simplify_sequences_with_reordering};
use crate::graphics::response::GraphicsDirective;
use crate::expr::Expr;
use crate::expr::calculus::{DerivativeEngine, DifferentiationFailure};

pub struct FunctionBuilder {
  /// The name of the function.
  name: String,
  /// Flags indicating miscellaneous properties about the function
  /// being built.
  flags: FunctionFlags,
  /// A predicate identifying the identity element of the function.
  /// See [`FunctionBuilder::set_identity`] for more details.
  identity_predicate: Box<dyn Fn(&Expr) -> bool + Send + Sync + 'static>,
  /// The rule for calculating the derivative of the function.
  derivative_rule: Option<Box<FunctionDeriv>>,
  /// Cases for ordinary, full evaluation of this function.
  cases: Vec<Box<FunctionCase<Expr>>>,
  /// Cases for evaluation of this function as part of the graphics
  /// subsystem.
  graphics_cases: Vec<Box<FunctionCase<GraphicsDirective>>>,
  /// Conditions on arguments which will trigger partial evaluation.
  /// If the function is marked as
  /// [`PERMITS_REORDERING`](FunctionFlags::PERMITS_REORDERING) then
  /// the arguments matched against these conditions need not be
  /// consecutive. If the function is NOT marked as
  /// [`PERMITS_FLATTENING`](FunctionFlags::PERMITS_FLATTENING), then
  /// this field is never used.
  partial_eval_predicates: Vec<Box<PartialEvalPredicate>>,
}

pub type FunctionCase<T> =
  dyn Fn(Vec<Expr>, &mut FunctionContext) -> FunctionCaseResult<T> + Send + Sync;

type PartialEvalPredicate = dyn Fn(&Expr) -> bool + Send + Sync;

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

impl FunctionBuilder {
  /// Constructs a new `FunctionBuilder` object for the function with
  /// the given name.
  pub fn new(name: impl Into<String>) -> Self {
    Self {
      name: name.into(),
      flags: FunctionFlags::default(),
      identity_predicate: Box::new(no_identity_value),
      derivative_rule: None,
      cases: Vec::new(),
      graphics_cases: Vec::new(),
      partial_eval_predicates: Vec::new(),
    }
  }

  /// Adds an evaluation case to `self`. This function is intended to
  /// be called in fluent style, and it returns `self` after
  /// modifications.
  pub fn add_case(mut self, case: Box<FunctionCase<Expr>>) -> Self {
    self.cases.push(case);
    self
  }

  /// Adds a graphics evaluation case to `self`. This function is
  /// intended to be called in fluent style, and it returns `self`
  /// after modifications.
  pub fn add_graphics_case(mut self, case: Box<FunctionCase<GraphicsDirective>>) -> Self {
    self.graphics_cases.push(case);
    self
  }

  /// Adds a partial evaluation condition to `self`. This function
  /// is intended to be called in fluent style, and it returns
  /// `self` after modifications.
  pub fn add_partial_eval_rule(mut self, predicate: Box<PartialEvalPredicate>) -> Self {
    self.partial_eval_predicates.push(predicate);
    self
  }

  /// Sets the predicate for recognizing identity values for the
  /// function being built. Modifies and returns `self`, to permit
  /// fluent-style calling.
  ///
  /// If `e` is an expression satisfying this predicate and `f` is the
  /// function being built, then `e` shall satisfy the following
  /// property: For all `x0...xi, y0...yj`, `f(x0, ..., xi, e, y0,
  /// ..., yj) = f(x0, ..., xi, y0, ..., yj)`.
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

  /// Enables the
  /// [`PERMITS_FLATTENING`](FunctionFlags::PERMITS_FLATTENING) flag
  /// for `self`.
  pub fn permit_flattening(mut self) -> Self {
    self.flags |= FunctionFlags::PERMITS_FLATTENING;
    self
  }

  /// Enables the
  /// [`PERMITS_REORDERING`](FunctionFlags::PERMITS_REORDERING) flag
  /// for `self`.
  pub fn permit_reordering(mut self) -> Self {
    self.flags |= FunctionFlags::PERMITS_REORDERING;
    self
  }

  /// Enables the [`IS_INVOLUTION`](FunctionFlags::IS_INVOLUTION) flag
  /// for `self`.
  pub fn mark_as_involution(mut self) -> Self {
    self.flags |= FunctionFlags::IS_INVOLUTION;
    self
  }

  /// Consumes `self` and builds it into a completed [`Function`]
  /// value.
  pub fn build(self) -> Function {
    let function_body = build_function_body(self.cases);
    let function_body =
      if self.flags.contains(FunctionFlags::PERMITS_FLATTENING) && !self.partial_eval_predicates.is_empty() {
        let name = self.name.clone();
        let flags = self.flags;
        let predicates = self.partial_eval_predicates;
        Box::new(move |args, context: &mut FunctionContext| {
          function_body(args, context).or_else(|args| {
            partial_eval(&name, args, flags, predicates.as_slice(), &function_body, context)
          })
        })
      } else {
        function_body
      };

    Function {
      name: self.name,
      flags: self.flags,
      identity_predicate: self.identity_predicate,
      derivative_rule: self.derivative_rule,
      body: function_body,
      graphics_body: build_function_body(self.graphics_cases),
    }
  }
}

fn build_function_body<T: 'static>(cases: Vec<Box<FunctionCase<T>>>) -> Box<FunctionImpl<T>> {
  if cases.is_empty() {
    // If there are no cases (for instance, `graphics_body` for a
    // function which is not used in graphics), return a much simpler
    // closure, for efficiency reasons.
    return Box::new(|args, _| Err(args));
  }

  Box::new(move |mut args, context: &mut FunctionContext| {
    for case in &cases {
      match case(args, context) {
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

fn partial_eval(
  function_name: &str,
  mut args: Vec<Expr>,
  flags: FunctionFlags,
  predicates: &[Box<PartialEvalPredicate>],
  function_body: &FunctionImpl<Expr>,
  context: &mut FunctionContext,
) -> Result<Expr, Vec<Expr>> {
  assert!(flags.contains(FunctionFlags::PERMITS_FLATTENING));
  let mut body = |args| {
    match function_body(args, context) {
      Ok(res) => vec![res],
      Err(args) => args,
    }
  };

  for pred in predicates {
    args = if flags.contains(FunctionFlags::PERMITS_REORDERING) {
      simplify_sequences_with_reordering(args, pred, &mut body)
    } else {
      simplify_sequences(args, pred, &mut body)
    };
  }
  if args.len() == 1 {
    let [arg] = args.try_into().unwrap();
    Ok(arg)
  } else {
    Ok(Expr::call(function_name, args))
  }
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
