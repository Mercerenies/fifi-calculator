
//! Functions for performing symbolic manipulation.

use crate::expr::Expr;
use crate::expr::var::Var;
use crate::expr::function::Function;
use crate::expr::function::builder::{self, FunctionBuilder};
use crate::expr::function::table::FunctionTable;
use crate::expr::simplifier::Simplifier;
use crate::expr::simplifier::error::SimplifierError;
use crate::expr::number::ComplexLike;
use crate::expr::algebra::newton::{NewtonRaphsonFunction, NewtonRaphsonMethod, NewtonRaphsonError};
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
/// self-referencing. That is, it's meaningful to replace `x` with `x
/// + 1` using this function, since the `x` on the right-hand side
/// will not get recursively substituted.
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
      builder::arity_three().of_types(prisms::expr_to_equation(), prisms::ExprToVar, prisms::ExprToComplex)
        .and_then(|equation, var, initial_guess, ctx| {
          // TODO: Consider how possible it is to clean up the
          // clone()s here.
          let expr = Expr::call("-", vec![equation.left.clone(), equation.right.clone()]);
          match find_root(expr, var.clone(), initial_guess.clone(), ctx.function_table, ctx.simplifier) {
            Ok(expr) => Ok(expr),
            Err(err) => {
              ctx.errors.push(SimplifierError::new("find_root", err));
              Err((equation, var, initial_guess))
            }
          }
        })
    )
    // Find root of arbitrary expression
    .add_case(
      builder::arity_three().of_types(Identity, prisms::ExprToVar, prisms::ExprToComplex)
        .and_then(|expr, var, initial_guess, ctx| {
          // TODO: Consider how possible it is to clean up the
          // clone()s here.
          match find_root(expr.clone(), var.clone(), initial_guess.clone(), ctx.function_table, ctx.simplifier) {
            Ok(expr) => Ok(expr),
            Err(err) => {
              ctx.errors.push(SimplifierError::new("find_root", err));
              Err((expr, var, initial_guess))
            }
          }
        })
    )
    .build()
}

fn find_root(
  expr: Expr,
  var: Var,
  initial_guess: ComplexLike,
  table: &FunctionTable,
  simplifier: &dyn Simplifier,
) -> Result<Expr, NewtonRaphsonError> {
  // Since the result from Newton-Raphson is never an exact quantity
  // anyway, we don't want to misleadingly provide rational results
  // with ludicrously large numerators and denominators, so go ahead
  // and force the whole computation to be inexact.
  let initial_guess = initial_guess.to_inexact();

  let algorithm = NewtonRaphsonMethod::default();
  let function = NewtonRaphsonFunction::from_expr(expr, var, table, simplifier)?;
  let root = algorithm.find_root(function, initial_guess)?;
  // TODO: Fallback methods in Newton-Raphson fails to find a derivative.
  Ok(root.into_expr())
}
