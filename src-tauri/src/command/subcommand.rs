
use crate::errorlist::ErrorList;
use crate::expr::Expr;
use crate::expr::simplifier::{Simplifier, SimplifierContext};
use crate::expr::simplifier::error::{SimplifierError, ArityError};
use crate::mode::calculation::CalculationMode;

/// A subcommand is a simplified command which merely takes arguments
/// and applies some function. This is usually applied in the context
/// of a vector operation, such as a fold or a map, which needs to
/// call some other function as part of its operation.
///
/// A subcommand consists of a function to apply and a specified
/// arity.
pub struct Subcommand<'a> {
  function: Box<SubcommandFunction<'a>>,
  arity: usize,
}

type SubcommandFunction<'a> =
  dyn Fn(Vec<Expr>) -> Expr + 'a;

impl<'a> Subcommand<'a> {
  /// Constructs a new subcommand from the function and specified
  /// arity. The function may assume that its argument vector has the
  /// requested arity.
  pub fn new<F>(arity: usize, function: F) -> Self
  where F: Fn(Vec<Expr>) -> Expr + 'a {
    Self {
      function: Box::new(function),
      arity,
    }
  }

  pub fn arity(&self) -> usize {
    self.arity
  }

  /// Invokes the function indicated by this subcommand on the
  /// arguments given. Returns an error in case of argument arity
  /// mismatch.
  pub fn try_call(
    &self,
    args: Vec<Expr>,
    simplifier: &dyn Simplifier,
    calculation_mode: CalculationMode,
    errors: &mut ErrorList<SimplifierError>,
  ) -> Result<Expr, ArityError> {
    if args.len() != self.arity {
      return Err(ArityError { expected: self.arity, actual: args.len() });
    }

    let mut simplifier_context = SimplifierContext {
      base_simplifier: simplifier,
      calculation_mode,
      errors,
    };
    let expr = (self.function)(args);
    let expr = simplifier.simplify_expr(expr, &mut simplifier_context);
    Ok(expr)
  }

  /// Invokes the function indicated by this subcommand on the
  /// arguments given. Panics in case of arity mismatch.
  pub fn call_or_panic(
    &self,
    args: Vec<Expr>,
    simplifier: & dyn Simplifier,
    calculation_mode: CalculationMode,
    errors: &mut ErrorList<SimplifierError>,
  ) -> Expr {
    self.try_call(args, simplifier, calculation_mode, errors).unwrap()
  }
}

impl Subcommand<'static> {
  /// Constructs a new subcommand whose behavior is to call a given
  /// function in the expression-language with the arguments.
  pub fn named(arity: usize, function_name: impl Into<String>) -> Self {
    let function_name = function_name.into();
    Self::new(arity, move |args| {
      Expr::call(function_name.clone(), args)
    })
  }
}

#[cfg(test)]
pub(crate) mod test_utils {
  use super::*;
  use crate::expr::function::table::FunctionTable;
  use crate::expr::simplifier::default_simplifier;
  use crate::expr::function::library::build_function_table;

  use once_cell::sync::Lazy;

  pub fn try_call(
    subcommand: &Subcommand,
    args: Vec<Expr>,
  ) -> Result<(Expr, ErrorList<SimplifierError>), ArityError> {
    static FUNCTION_TABLE: Lazy<FunctionTable> = Lazy::new(build_function_table);
    let simplifier = default_simplifier(&FUNCTION_TABLE);
    let calculation_mode = CalculationMode::default();
    let mut errors = ErrorList::new();
    let expr = subcommand.try_call(args, simplifier.as_ref(), calculation_mode, &mut errors)?;
    Ok((expr, errors))
  }
}

#[cfg(test)]
mod tests {
  use super::*;
  use super::test_utils::try_call;

  #[test]
  fn test_named_subcommand() {
    let subcommand = Subcommand::named(2, "test");

    let (expr, errors) = try_call(&subcommand, vec![Expr::from(1), Expr::from(2)]).unwrap();
    assert!(errors.is_empty());
    assert_eq!(expr, Expr::call("test", vec![Expr::from(1), Expr::from(2)]));

    let err = try_call(&subcommand, vec![Expr::from(1), Expr::from(2), Expr::from(3)]).unwrap_err();
    assert_eq!(err, ArityError { expected: 2, actual: 3 });

    let err = try_call(&subcommand, vec![Expr::from(1)]).unwrap_err();
    assert_eq!(err, ArityError { expected: 2, actual: 1 });
  }
}
