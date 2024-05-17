
use crate::expr::Expr;
use crate::expr::simplifier::error::SimplifierError;
use crate::errorlist::ErrorList;

use std::fmt::{self, Formatter, Debug};

/// For the purposes of
/// [`FunctionEvaluator`](super::FunctionEvaluator), a function is a
/// method of taking a vector of expressions as arguments and
/// evaluating them down to one expression, with the possibility of
/// failing and returning the original vector of expressions back.
pub struct Function {
  name: String,
  body: Box<FunctionImpl>,
}

type FunctionImpl =
  dyn Fn(Vec<Expr>, &mut ErrorList<SimplifierError>) -> Result<Expr, Vec<Expr>> + Send + Sync;

impl Function {
  /// Directly constructs a function with the given name and implementation.
  ///
  /// Note that most outside callers will prefer to use the [builder
  /// API](super::builder) instead.
  pub fn new<S, F>(name: S, body: F) -> Function
  where S: Into<String>,
        F: Fn(Vec<Expr>, &mut ErrorList<SimplifierError>) -> Result<Expr, Vec<Expr>>,
        F: Send + Sync + 'static {
    Function { name: name.into(), body: Box::new(body) }
  }

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
