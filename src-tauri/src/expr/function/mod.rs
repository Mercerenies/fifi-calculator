
pub mod builder;
pub mod flags;
pub mod library;
pub mod table;

use flags::FunctionFlags;
use table::FunctionTable;
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
  flags: FunctionFlags,
  identity_predicate: Box<dyn Fn(&Expr) -> bool + Send + Sync + 'static>,
  body: Box<FunctionImpl>,
}

pub struct FunctionContext<'a, 'b> {
  pub errors: &'a mut ErrorList<SimplifierError>,
  pub function_table: &'b FunctionTable,
  _private: (),
}

type FunctionImpl =
  dyn Fn(Vec<Expr>, FunctionContext) -> Result<Expr, Vec<Expr>> + Send + Sync;

impl Function {
  /// The function's name.
  pub fn name(&self) -> &str {
    &self.name
  }

  /// The property-based flags set on this function.
  pub fn flags(&self) -> FunctionFlags {
    self.flags
  }

  pub fn is_identity(&self, arg: &Expr) -> bool {
    (self.identity_predicate)(arg)
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
    function_table: &FunctionTable,
  ) -> Result<Expr, Vec<Expr>> {
    let context = FunctionContext { errors, function_table, _private: () };
    (self.body)(args, context)
  }
}

impl Debug for Function {
  fn fmt(&self, f: &mut Formatter) -> Result<(), fmt::Error> {
    write!(f, "Function {{ name: {:?}, flags: {:?}, body: ... }}", self.name, self.flags)
  }
}

fn no_identity_value(_: &Expr) -> bool {
  false
}
