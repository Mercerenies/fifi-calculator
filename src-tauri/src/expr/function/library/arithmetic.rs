
//! Basic arithmetic function evaluation rules.

use crate::expr::Expr;
use crate::expr::function::Function;
use crate::expr::function::table::FunctionTable;
use crate::expr::function::builder::{self, FunctionBuilder, FunctionCaseResult};
use crate::expr::prisms::{ExprToNumber, ExprToComplex};
use crate::expr::number::{Number, ComplexNumber, pow_real, pow_complex, pow_complex_to_real};
use crate::expr::simplifier::error::SimplifierError;

use num::{Zero, One};

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
    .set_derivative(
      |args, engine| {
        let args = engine.differentiate_each(args)?;
        Ok(Expr::call("+", args))
      }
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
    .set_derivative(
      builder::arity_two_deriv("-", |arg1, arg2, engine| {
        Ok(Expr::call("-", vec![engine.differentiate(arg1)?, engine.differentiate(arg2)?]))
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
      Box::new(|args, _context| {
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
    .set_derivative(
      |args, engine| {
        let mut final_terms = Vec::with_capacity(args.len());
        for i in 0..args.len() {
          let mut args = args.clone();
          args[i].mutate_failable(|e| engine.differentiate(e))?;
          final_terms.push(Expr::call("*", args));
        }
        Ok(Expr::call("+", args))
      }
    )
    .build()
}

pub fn division() -> Function {
  FunctionBuilder::new("/")
    .add_case(
      // Real number division
      builder::arity_two().both_of_type(ExprToNumber).and_then(|arg1, arg2, context| {
        if arg2.is_zero() {
          context.errors.push(SimplifierError::division_by_zero("/"));
          return Err((arg1, arg2));
        }
        let quotient = arg1 / arg2;
        Ok(Expr::from(quotient))
      })
    )
    .add_case(
      // Complex number division
      builder::arity_two().both_of_type(ExprToComplex).and_then(|arg1, arg2, context| {
        if arg2.is_zero() {
          context.errors.push(SimplifierError::division_by_zero("/"));
          return Err((arg1, arg2));
        }
        let quotient = ComplexNumber::from(arg1) / ComplexNumber::from(arg2);
        Ok(Expr::from(quotient))
      })
    )
    .set_derivative(
      builder::arity_two_deriv("/", |arg1, arg2, engine| {
        let arg1_deriv = engine.differentiate(arg1.clone())?;
        let arg2_deriv = engine.differentiate(arg2.clone())?;
        let final_numerator = Expr::call("-", vec![
          Expr::call("*", vec![arg1_deriv, arg2.clone()]),
          Expr::call("*", vec![arg1, arg2_deriv]),
        ]);
        let final_denominator = Expr::call("^", vec![arg2, Expr::from(Number::from(2))]);
        Ok(Expr::call("/", vec![final_numerator, final_denominator]))
      })
    )
    .build()
}

pub fn power() -> Function {
  FunctionBuilder::new("^")
    .add_case(
      // Real number power function
      builder::arity_two().both_of_type(ExprToNumber).and_then(|arg1, arg2, context| {
        if arg1.is_zero() && arg2.is_zero() {
          context.errors.push(SimplifierError::zero_to_zero_power("^"));
          return Err((arg1, arg2));
        }
        if arg1.is_zero() && arg2 < Number::zero() {
          context.errors.push(SimplifierError::division_by_zero("^"));
          return Err((arg1, arg2));
        }
        let power = pow_real(arg1, arg2);
        Ok(Expr::from(power))
      })
    )
    .add_case(
      // Complex-to-real number power function
      builder::arity_two().of_types(ExprToComplex, ExprToNumber).and_then(|arg1, arg2, context| {
        if arg1.is_zero() && arg2.is_zero() {
          context.errors.push(SimplifierError::zero_to_zero_power("^"));
          return Err((arg1, arg2));
        }
        if arg1.is_zero() && arg2 < Number::zero() {
          context.errors.push(SimplifierError::division_by_zero("^"));
          return Err((arg1, arg2));
        }
        let power = pow_complex_to_real(arg1.into(), arg2);
        Ok(Expr::from(power))
      })
    )
    .add_case(
      // Complex number power function
      builder::arity_two().of_types(ExprToComplex, ExprToComplex).and_then(|arg1, arg2, context| {
        if arg1.is_zero() && arg2.is_zero() {
          context.errors.push(SimplifierError::zero_to_zero_power("^"));
          return Err((arg1, arg2));
        }
        if arg1.is_zero() {
          // Arguably, a positive real exponent which happens to be
          // represented as a complex number can simplify here, but
          // that's getting so far into the weeds. Just bail out if
          // arg1 == 0 in general.
          context.errors.push(SimplifierError::division_by_zero("^"));
          return Err((arg1, arg2));
        }
        let power = pow_complex(arg1.into(), arg2.into());
        Ok(Expr::from(power))
      })
    )
    .build()
  // TODO: Derivative
}

pub fn modulo() -> Function {
  FunctionBuilder::new("%")
    .add_case(
      // Real modulo
      builder::arity_two().both_of_type(ExprToNumber).and_then(|arg1, arg2, context| {
        if arg2.is_zero() {
          context.errors.push(SimplifierError::division_by_zero("%"));
          return Err((arg1, arg2));
        }
        Ok(Expr::from(arg1 % arg2))
      })
    )
    .add_case(
      // Trap case: Complex numbers
      builder::arity_two().both_of_type(ExprToComplex).and_then(|arg1, arg2, context| {
        context.errors.push(SimplifierError::expected_real("%"));
        Err((arg1, arg2))
      })
    )
    .build()
  // TODO: Derivative
}

pub fn floor_division() -> Function {
  FunctionBuilder::new("div")
    .add_case(
      // Real floor div
      builder::arity_two().both_of_type(ExprToNumber).and_then(|arg1, arg2, context| {
        if arg2.is_zero() {
          context.errors.push(SimplifierError::division_by_zero("div"));
          return Err((arg1, arg2));
        }
        Ok(Expr::from(arg1.div_floor(&arg2)))
      })
    )
    .add_case(
      // Trap case: Complex numbers
      builder::arity_two().both_of_type(ExprToComplex).and_then(|arg1, arg2, context| {
        context.errors.push(SimplifierError::expected_real("div"));
        Err((arg1, arg2))
      })
    )
    .build()
}

pub fn arithmetic_negate() -> Function {
  FunctionBuilder::new("negate")
    .add_case(
      // Real number negation
      builder::arity_one().of_type(ExprToNumber).and_then(|arg, _| {
        Ok(Expr::from(- arg))
      })
    )
    .add_case(
      // Complex number negation
      builder::arity_one().of_type(ExprToComplex).and_then(|arg, _| {
        let arg = ComplexNumber::from(arg);
        Ok(Expr::from(- arg))
      })
    )
    .build()
  // TODO: Derivative
}
