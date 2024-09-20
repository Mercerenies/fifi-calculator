
pub mod builder;
pub mod distributive;
pub mod flags;
pub mod library;
pub mod partial;
pub mod table;

use flags::FunctionFlags;
use table::FunctionTable;
use crate::mode::calculation::CalculationMode;
use crate::graphics::response::GraphicsDirective;
use crate::expr::Expr;
use crate::expr::simplifier::Simplifier;
use crate::expr::simplifier::error::SimplifierError;
use crate::expr::calculus::{DerivativeEngine, DifferentiationFailure, DifferentiationError};
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
  derivative_rule: Option<Box<FunctionDeriv>>,
  body: Box<FunctionImpl<Expr>>,
  graphics_body: Box<FunctionImpl<GraphicsDirective>>,
}

pub struct FunctionContext<'a, 'b, 'c> {
  pub errors: &'a mut ErrorList<SimplifierError>,
  pub simplifier: &'b dyn Simplifier,
  pub function_table: &'c FunctionTable,
  pub calculation_mode: CalculationMode,
  _private: (),
}

type FunctionImpl<T> =
  dyn Fn(Vec<Expr>, &mut FunctionContext) -> Result<T, Vec<Expr>> + Send + Sync;

type FunctionDeriv =
  dyn Fn(Vec<Expr>, &DerivativeEngine) -> Result<Expr, DifferentiationFailure> + Send + Sync;

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
    simplifier: &dyn Simplifier,
    function_table: &FunctionTable,
    calculation_mode: CalculationMode,
  ) -> Result<Expr, Vec<Expr>> {
    let mut context = FunctionContext { errors, simplifier, function_table, calculation_mode, _private: () };
    (self.body)(args, &mut context)
  }

  /// Calls the function as part of the graphics subsystem.
  pub fn call_for_graphics(
    &self,
    args: Vec<Expr>,
    errors: &mut ErrorList<SimplifierError>,
    simplifier: &dyn Simplifier,
    function_table: &FunctionTable,
    calculation_mode: CalculationMode,
  ) -> Result<GraphicsDirective, Vec<Expr>> {
    let mut context = FunctionContext { errors, simplifier, function_table, calculation_mode, _private: () };
    (self.graphics_body)(args, &mut context)
  }

  pub fn differentiate(
    &self,
    args: Vec<Expr>,
    engine: &DerivativeEngine,
  ) -> Result<Expr, DifferentiationFailure> {
    let Some(derivative_rule) = &self.derivative_rule else {
      return Err(engine.error(DifferentiationError::UnknownDerivative(self.name().to_owned())));
    };
    derivative_rule(args, engine)
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
