
//! Basic arithmetic function evaluation rules.

use crate::expr::Expr;
use crate::expr::var::Var;
use crate::expr::function::Function;
use crate::expr::function::table::FunctionTable;
use crate::expr::function::builder::{self, FunctionBuilder, FunctionCaseResult};
use crate::expr::prisms::{ExprToNumber, ExprToComplex, must_be_var};
use crate::expr::number::{Number, ComplexNumber, pow_real, pow_complex, pow_complex_to_real};
use crate::expr::simplifier::error::SimplifierError;

use num::{Zero, One};

use std::f64::consts;

pub fn append_arithmetic_functions(table: &mut FunctionTable) {
  table.insert(addition());
  table.insert(subtraction());
  table.insert(multiplication());
  table.insert(division());
  table.insert(power());
  table.insert(modulo());
  table.insert(floor_division());
  table.insert(arithmetic_negate());
}

pub fn addition() -> Function {
  FunctionBuilder::new("+")
    .permit_flattening()
    .set_identity(Expr::is_zero)
    .add_case(
      // Unary simplification
      builder::arity_one().and_then(|arg, _| {
        Ok(arg)
      })
    )
    .add_case(
      // Real number addition
      builder::any_arity().of_type(ExprToNumber).and_then(|args, _| {
        let sum = args.into_iter().reduce(|a, b| a + b).unwrap_or(Number::zero());
        Ok(Expr::from(sum))
      })
    )
    .add_case(
      // Complex number addition
      builder::any_arity().of_type(ExprToComplex).and_then(|args, _| {
        let sum = args.into_iter()
          .map(ComplexNumber::from)
          .reduce(|a, b| a + b)
          .unwrap_or(ComplexNumber::zero());
        Ok(Expr::from(sum))
      })
    )
    .build()
}

pub fn subtraction() -> Function {
  FunctionBuilder::new("-")
    .add_case(
      // Real number subtraction
      builder::arity_two().both_of_type(ExprToNumber).and_then(|arg1, arg2, _| {
        let difference = arg1 - arg2;
        Ok(Expr::from(difference))
      })
    )
    .add_case(
      // Complex number subtraction
      builder::arity_two().both_of_type(ExprToComplex).and_then(|arg1, arg2, _| {
        let difference = ComplexNumber::from(arg1) - ComplexNumber::from(arg2);
        Ok(Expr::from(difference))
      })
    )
    .build()
}

pub fn multiplication() -> Function {
  FunctionBuilder::new("*")
    .permit_flattening()
    .set_identity(Expr::is_one)
    .add_case(
      // Unary simplification
      builder::arity_one().and_then(|arg, _| {
        Ok(arg)
      })
    )
    .add_case(
      // Multiplication by zero
      Box::new(|args, _errors| {
        // TODO: The manual construction of FunctionCase and explicit
        // FunctionCaseResults here are less than ideal. Can we make a
        // builder for this?
        let contains_zero = args.iter().any(Expr::is_zero);
        if contains_zero {
          FunctionCaseResult::Success(Expr::zero())
        } else {
          FunctionCaseResult::NoMatch(args)
        }
      })
    )
    .add_case(
      // Real number multiplication
      builder::any_arity().of_type(ExprToNumber).and_then(|args, _| {
        let product = args.into_iter().reduce(|a, b| a * b).unwrap_or(Number::one());
        Ok(Expr::from(product))
      })
    )
    .add_case(
      // Complex number multiplication
      builder::any_arity().of_type(ExprToComplex).and_then(|args, _| {
        let product = args.into_iter()
          .map(ComplexNumber::from)
          .reduce(|a, b| a * b)
          .unwrap_or(ComplexNumber::one());
        Ok(Expr::from(product))
      })
    )
    .build()
}

pub fn division() -> Function {
  FunctionBuilder::new("/")
    .add_case(
      // Real number division
      builder::arity_two().both_of_type(ExprToNumber).and_then(|arg1, arg2, errors| {
        if arg2.is_zero() {
          errors.push(SimplifierError::division_by_zero("/"));
          return Err((arg1, arg2));
        }
        let quotient = arg1 / arg2;
        Ok(Expr::from(quotient))
      })
    )
    .add_case(
      // Complex number division
      builder::arity_two().both_of_type(ExprToComplex).and_then(|arg1, arg2, errors| {
        if arg2.is_zero() {
          errors.push(SimplifierError::division_by_zero("/"));
          return Err((arg1, arg2));
        }
        let quotient = ComplexNumber::from(arg1) / ComplexNumber::from(arg2);
        Ok(Expr::from(quotient))
      })
    )
    .build()
}

pub fn power() -> Function {
  let e = Var::new("e").unwrap();
  FunctionBuilder::new("^")
    .add_case(
      // Real number power function
      builder::arity_two().both_of_type(ExprToNumber).and_then(|arg1, arg2, errors| {
        if arg1.is_zero() && arg2.is_zero() {
          errors.push(SimplifierError::zero_to_zero_power("^"));
          return Err((arg1, arg2));
        }
        if arg1.is_zero() && arg2 < Number::zero() {
          errors.push(SimplifierError::division_by_zero("^"));
          return Err((arg1, arg2));
        }
        let power = pow_real(arg1, arg2);
        Ok(Expr::from(power))
      })
    )
    .add_case(
      // Complex-to-real number power function
      builder::arity_two().of_types(ExprToComplex, ExprToNumber).and_then(|arg1, arg2, errors| {
        if arg1.is_zero() && arg2.is_zero() {
          errors.push(SimplifierError::zero_to_zero_power("^"));
          return Err((arg1, arg2));
        }
        if arg1.is_zero() && arg2 < Number::zero() {
          errors.push(SimplifierError::division_by_zero("^"));
          return Err((arg1, arg2));
        }
        let power = pow_complex_to_real(arg1.into(), arg2);
        Ok(Expr::from(power))
      })
    )
    .add_case(
      // Complex number power function
      builder::arity_two().of_types(ExprToComplex, ExprToComplex).and_then(|arg1, arg2, errors| {
        if arg1.is_zero() && arg2.is_zero() {
          errors.push(SimplifierError::zero_to_zero_power("^"));
          return Err((arg1, arg2));
        }
        if arg1.is_zero() {
          // Arguably, a positive real exponent which happens to be
          // represented as a complex number can simplify here, but
          // that's getting so far into the weeds. Just bail out if
          // arg1 == 0 in general.
          errors.push(SimplifierError::division_by_zero("^"));
          return Err((arg1, arg2));
        }
        let power = pow_complex(arg1.into(), arg2.into());
        Ok(Expr::from(power))
      })
    )
    .add_case(
      // e^{known real number} case
      builder::arity_two().of_types(must_be_var(e.clone()), ExprToNumber).and_then(|_unit, arg2, _errors| {
        // Some simplifications and rules introduce expressions of the
        // form `e^x` (for some `x`). In general, we'd like to treat
        // this a lot like a function and leave the `e` unevaluated,
        // but if `x` is a known constant, then go ahead and give us
        // the (inexact) result.
        let e = Number::from(consts::E);
        let power = pow_real(e, arg2);
        Ok(Expr::from(power))
      })
    )
    .add_case(
      // e^{known complex number} case
      builder::arity_two().of_types(must_be_var(e), ExprToComplex).and_then(|_unit, arg2, _errors| {
        // Some simplifications and rules introduce expressions of the
        // form `e^x` (for some `x`). In general, we'd like to treat
        // this a lot like a function and leave the `e` unevaluated,
        // but if `x` is a known constant, then go ahead and give us
        // the (inexact) result.
        let e = ComplexNumber::from_real(Number::from(consts::E));
        let power = pow_complex(e, arg2.into());
        Ok(Expr::from(power))
      })
    )
    .build()
}

pub fn modulo() -> Function {
  FunctionBuilder::new("%")
    .add_case(
      // Real modulo
      builder::arity_two().both_of_type(ExprToNumber).and_then(|arg1, arg2, errors| {
        if arg2.is_zero() {
          errors.push(SimplifierError::division_by_zero("%"));
          return Err((arg1, arg2));
        }
        Ok(Expr::from(arg1 % arg2))
      })
    )
    .add_case(
      // Trap case: Complex numbers
      builder::arity_two().both_of_type(ExprToComplex).and_then(|arg1, arg2, errors| {
        errors.push(SimplifierError::expected_real("%"));
        Err((arg1, arg2))
      })
    )
    .build()
}

pub fn floor_division() -> Function {
  FunctionBuilder::new("div")
    .add_case(
      // Real floor div
      builder::arity_two().both_of_type(ExprToNumber).and_then(|arg1, arg2, errors| {
        if arg2.is_zero() {
          errors.push(SimplifierError::division_by_zero("div"));
          return Err((arg1, arg2));
        }
        Ok(Expr::from(arg1.div_floor(&arg2)))
      })
    )
    .add_case(
      // Trap case: Complex numbers
      builder::arity_two().both_of_type(ExprToComplex).and_then(|arg1, arg2, errors| {
        errors.push(SimplifierError::expected_real("div"));
        Err((arg1, arg2))
      })
    )
    .build()
}

pub fn arithmetic_negate() -> Function {
  FunctionBuilder::new("negate")
    .add_case(
      // Real number addition
      builder::arity_one().of_type(ExprToNumber).and_then(|arg, _| {
        Ok(Expr::from(- arg))
      })
    )
    .add_case(
      // Complex number addition
      builder::arity_one().of_type(ExprToComplex).and_then(|arg, _| {
        let arg = ComplexNumber::from(arg);
        Ok(Expr::from(- arg))
      })
    )
    .build()
}
