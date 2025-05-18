
//! Functions for performing symbolic manipulation.

use crate::expr::Expr;
use crate::expr::var::Var;
use crate::expr::function::Function;
use crate::expr::function::builder::{self, FunctionBuilder};
use crate::expr::function::table::FunctionTable;
use crate::expr::simplifier::Simplifier;
use crate::expr::simplifier::error::SimplifierError;
use crate::expr::number::{Number, ComplexLike};
use crate::expr::interval::Interval;
use crate::expr::algebra::root_finding::{RootFindingInput, expr_to_root_finding_input};
use crate::expr::algebra::root_finding::newton::{NewtonRaphsonFunction, NewtonRaphsonMethod, NewtonRaphsonError};
use crate::expr::algebra::root_finding::secant::{SecantMethodFunction, SecantMethod, SecantMethodError};
use crate::expr::algebra::root_finding::bisection::{BisectionFunction, BisectionMethod, BisectionError};
use crate::expr::prisms;
use crate::util::prism::Identity;

pub fn append_symbolic_functions(table: &mut FunctionTable) {
  table.insert(substitute_function());
  table.insert(find_root_function());
}

/// Replaces all instances of the needle variable with the given
/// replacement expression in the haystack.
///
/// It is NOT an error for the variable to be absent from the target
/// stack expression. In that case, the stack value is unchanged. This
/// command is also inherently single-pass, so a substitution can be
/// self-referencing. That is, it is meaningful and well-defined to
/// replace `x` with `x + 1` using this function, since the `x` on the
/// right-hand side will not get recursively substituted.
pub fn substitute_function() -> Function {
  FunctionBuilder::new("substitute")
    .add_case(
      builder::arity_three().of_types(Identity, prisms::ExprToVar, Identity)
        .and_then(|haystack, needle, replacement, _| {
          Ok(haystack.substitute_var(needle, replacement))
        })
    )
    .build()
}

/// Attempts to (numerically) find a root for the given function.
pub fn find_root_function() -> Function {
  FunctionBuilder::new("find_root")
    // Find root of equation
    .add_case(
      builder::arity_three().of_types(prisms::expr_to_equation(), prisms::ExprToVar, expr_to_root_finding_input())
        .and_then(|equation, var, input, ctx| {
          // TODO: Consider how possible it is to clean up the
          // clone()s here.
          let expr = Expr::call("-", vec![equation.left.clone(), equation.right.clone()]);
          match find_root(expr, var.clone(), input.clone(), ctx.function_table, ctx.simplifier) {
            Ok(expr) => Ok(expr),
            Err(err) => {
              ctx.errors.push(SimplifierError::new("find_root", err));
              Err((equation, var, input))
            }
          }
        })
    )
    // Find root of arbitrary expression
    .add_case(
      builder::arity_three().of_types(Identity, prisms::ExprToVar, expr_to_root_finding_input())
        .and_then(|expr, var, input, ctx| {
          // TODO: Consider how possible it is to clean up the
          // clone()s here.
          match find_root(expr.clone(), var.clone(), input.clone(), ctx.function_table, ctx.simplifier) {
            Ok(expr) => Ok(expr),
            Err(err) => {
              ctx.errors.push(SimplifierError::new("find_root", err));
              Err((expr, var, input))
            }
          }
        })
    )
    .build()
}

fn find_root(
  expr: Expr,
  var: Var,
  input: RootFindingInput,
  table: &FunctionTable,
  simplifier: &dyn Simplifier,
) -> Result<Expr, anyhow::Error> {
  const OFFSET_EPSILON: f64 = 0.01;

  match input {
    RootFindingInput::Complex(complex_input) => {
      // Complex input; MUST use Newton-Raphson
      let function = NewtonRaphsonFunction::from_expr(expr, var, table, simplifier)
        .map_err(|failure| failure.error)?;
      let result = find_root_newton(&function, ComplexLike::Complex(complex_input))?;
      Ok(result)
    }
    RootFindingInput::Real(real_input) => {
      // Real input; try Newton-Raphson but fall back to Secant Method
      // if there's no derivative.
      match NewtonRaphsonFunction::from_expr(expr, var.clone(), table, simplifier) {
        Ok(function) => {
          let result = find_root_newton(&function, ComplexLike::Real(real_input))?;
          Ok(result)
        }
        Err(err) => {
          let expr = err.original_expr;
          let function = SecantMethodFunction::from_expr(expr, var, simplifier);
          let result = find_root_secant(&function, real_input.clone(), real_input + Number::from(OFFSET_EPSILON))?;
          Ok(result)
        }
      }
    }
    RootFindingInput::PairOfReals(pair) => {
      // Two real numbers as input. Always use Secant Method.
      let function = SecantMethodFunction::from_expr(expr, var, simplifier);
      let result = find_root_secant(&function, pair.0, pair.1)?;
      Ok(result)
    }
    RootFindingInput::Interval(raw_interval) => {
      let interval = Interval::from(raw_interval);
      let function = BisectionFunction::from_expr(expr, var, simplifier);
      let result = find_root_bisection(&function, interval)?;
      Ok(result)
    }
  }
}

fn find_root_newton(
  function: &NewtonRaphsonFunction,
  initial_guess: ComplexLike,
) -> Result<Expr, NewtonRaphsonError> {
  // Since the result from Newton-Raphson is never an exact quantity
  // anyway, we don't want to misleadingly provide rational results
  // with ludicrously large numerators and denominators, so go ahead
  // and force the whole computation to be inexact.
  let initial_guess = initial_guess.to_inexact();

  let algorithm = NewtonRaphsonMethod::default();
  let root = algorithm.find_root(function, initial_guess)?;
  Ok(root.into_expr())
}

fn find_root_secant(
  function: &SecantMethodFunction,
  initial_guess1: Number,
  initial_guess2: Number,
) -> Result<Expr, SecantMethodError> {
  // Since the result from Secant Method is never an exact quantity
  // anyway, we don't want to misleadingly provide rational results
  // with ridiculously large numerators and denominators, so go ahead
  // and force the whole computation to be inexact.
  let initial_guess1 = initial_guess1.to_inexact();
  let initial_guess2 = initial_guess2.to_inexact();

  let algorithm = SecantMethod::default();
  let root = algorithm.find_root(function, initial_guess1, initial_guess2)?;
  Ok(root.into_expr())
}

fn find_root_bisection(
  function: &BisectionFunction,
  interval: Interval<Number>,
) -> Result<Expr, BisectionError> {
  // Since the result from Bisection Method is never an exact quantity
  // anyway, we don't want to misleadingly provide rational results
  // with ridiculously large numerators and denominators, so go ahead
  // and force the whole computation to be inexact.
  let (left_bound, right_bound) = interval.into_extremes();
  let left_bound = left_bound.to_inexact();
  let right_bound = right_bound.to_inexact();

  let algorithm = BisectionMethod::default();
  let root = algorithm.find_root(function, left_bound, right_bound)?;
  Ok(root.into_expr())
}
