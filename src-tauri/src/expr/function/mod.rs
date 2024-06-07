
pub mod builder;
pub mod library;
pub mod table;

use crate::expr::Expr;
use crate::expr::simplifier::error::SimplifierError;
use crate::errorlist::ErrorList;

use std::fmt::{self, Formatter, Debug};

/// A mathematical function known to the calculator engine. A function
/// contains zero or more evaluation rules. Functions cannot be
/// constructed directly and must be constructed through the builder
/// API.
pub struct Function {
  name: String,
  body: Box<FunctionImpl>,
}

type FunctionImpl =
  dyn Fn(Vec<Expr>, &mut ErrorList<SimplifierError>) -> Result<Expr, Vec<Expr>> + Send + Sync;

impl Function {
  /// Directly constructs a function with the given name and implementation.
  fn new<S, F>(name: S, body: F) -> Function
  where S: Into<String>,
        F: Fn(Vec<Expr>, &mut ErrorList<SimplifierError>) -> Result<Expr, Vec<Expr>>,
        F: Send + Sync + 'static {
    Function { name: name.into(), body: Box::new(body) }
  }

  /// The function's name.
  pub fn name(&self) -> &str {
    &self.name
  }

  /// Calls the function, with the intent of fully evaluating it.
  ///
  /// If the function can be fully evaluated, this method returns
  /// `Ok`. If the function cannot be fully evaluated (for instance,
  /// because one or more of the arguments is a variable), this method
  /// returns `Err` with the original arguments. If the evaluation
  /// fails (for instance, due to a type error or division by zero),
  /// this method returns `Err` with the original arguments and pushes
  /// errors to the `errors` argument.
  pub fn call(
    &self,
    args: Vec<Expr>,
    errors: &mut ErrorList<SimplifierError>,
  ) -> Result<Expr, Vec<Expr>> {
    (self.body)(args, errors)
  }
}

impl Debug for Function {
  fn fmt(&self, f: &mut Formatter) -> Result<(), fmt::Error> {
    write!(f, "Function {{ name: \"{}\", body: ... }}", self.name)
  }
}
