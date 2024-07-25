
//! Evaluation rules for transcendental and trigonometric functions.

use crate::expr::Expr;
use crate::expr::simplifier::error::SimplifierError;
use crate::expr::function::Function;
use crate::expr::function::table::FunctionTable;
use crate::expr::function::builder::{self, FunctionBuilder};
use crate::expr::prisms::{self, expr_to_number, ExprToComplex};
use crate::expr::number::{Number, ComplexNumber, pow_real, pow_complex};
use crate::expr::algebra::infinity::InfiniteConstant;
use crate::expr::interval::Interval;

use num::{Zero, One};

use std::f64::consts;

pub fn append_transcendental_functions(table: &mut FunctionTable) {
  table.insert(natural_log());
  table.insert(logarithm());
  table.insert(exponent());
}

pub fn natural_log() -> Function {
  FunctionBuilder::new("ln")
    .add_case(
      // Natural logarithm of a positive real number
      builder::arity_one().of_type(prisms::expr_to_positive_number()).and_then(|arg, _| {
        Ok(Expr::from(Number::from(arg).ln()))
      })
    )
    .add_case(
      // Natural logarithm of a non-zero complex number
      builder::arity_one().of_type(prisms::ExprToComplex).and_then(|arg, ctx| {
        if arg.is_zero() {
          // TODO: Negative infinity
          ctx.errors.push(SimplifierError::custom_error("ln", "Expected non-zero complex number"));
          return Err(arg);
        }
        let arg = ComplexNumber::from(arg);
        Ok(Expr::from(arg.ln()))
      })
    )
    .add_case(
      // Natural logarithm of interval of positive reals
      builder::arity_one().of_type(prisms::expr_to_interval()).and_then(|arg, ctx| {
        if arg.left <= Number::zero() || arg.right <= Number::zero() {
          ctx.errors.push(SimplifierError::custom_error("ln", "Expected interval of positive reals"));
          return Err(arg);
        }
        let arg = Interval::from(arg);
        Ok(Expr::from(arg.map_monotone(|x| x.ln())))
      })
    )
    .add_case(
      // Natural logarithm of infinity
      builder::arity_one().of_type(prisms::ExprToInfinity).and_then(|arg, _| {
        if arg == InfiniteConstant::NotANumber {
          Ok(Expr::from(InfiniteConstant::NotANumber))
        } else {
          Ok(Expr::from(InfiniteConstant::PosInfinity))
        }
      })
    )
    .set_derivative(
      builder::arity_one_deriv("ln", |arg, engine| {
        let arg_deriv = engine.differentiate(arg.clone())?;
        Ok(Expr::call("/", vec![arg_deriv, arg]))
      })
    )
    .build()
}

pub fn logarithm() -> Function {
  FunctionBuilder::new("log")
    .add_case(
      // Arbitrary-base logarithm with positive real arguments
      builder::arity_two().both_of_type(prisms::expr_to_positive_number()).and_then(|arg, base, ctx| {
        if base.is_one() {
          ctx.errors.push(SimplifierError::division_by_zero("log"));
          return Err((arg, base));
        }
        let arg = Number::from(arg);
        let base = Number::from(base);
        Ok(Expr::from(arg.log(&base)))
      })
    )
    .add_case(
      // Arbitrary-base logarithm with non-zero complex arguments
      builder::arity_two().both_of_type(prisms::ExprToComplex).and_then(|arg, base, ctx| {
        // TODO: Infinity handling
        if arg.is_zero() || base.is_zero() {
          ctx.errors.push(SimplifierError::custom_error("log", "Expected non-zero complex arguments"));
          return Err((arg, base));
        }
        if base.is_one() {
          ctx.errors.push(SimplifierError::division_by_zero("log"));
          return Err((arg, base));
        }
        let arg = ComplexNumber::from(arg);
        let base = ComplexNumber::from(base);
        Ok(Expr::from(arg.ln() / base.ln()))
      })
    )
    .add_case(
      // Arbitrary-base logarithm of an interval with positive real base
      builder::arity_two().of_types(prisms::expr_to_interval(), prisms::expr_to_positive_number()).and_then(|arg, base, ctx| {
        if arg.left <= Number::zero() || arg.right <= Number::zero() {
          ctx.errors.push(SimplifierError::custom_error("log", "Expected interval of positive reals"));
          return Err((arg, base));
        }
        let base = Number::from(base);
        let arg = Interval::from(arg);
        Ok(Expr::from(arg.map_monotone(|x| x.log(&base))))
      })
    )
    .add_case(
      // Logarithm of complex with infinite base
      builder::arity_two().of_types(prisms::ExprToComplex, prisms::ExprToInfinity).and_then(|_arg, base, _| {
        if base == InfiniteConstant::NotANumber {
          Ok(Expr::from(InfiniteConstant::NotANumber))
        } else {
          Ok(Expr::zero())
        }
      })
    )
    .add_case(
      // Logarithm of infinity with complex base
      builder::arity_two().of_types(prisms::ExprToInfinity, prisms::ExprToComplex).and_then(|arg, _base, _| {
        if arg == InfiniteConstant::NotANumber {
          Ok(Expr::from(InfiniteConstant::NotANumber))
        } else {
          Ok(Expr::from(InfiniteConstant::PosInfinity))
        }
      })
    )
    .add_case(
      // Logarithm of infinity with infinite base (indeterminate)
      builder::arity_two().both_of_type(prisms::ExprToInfinity).and_then(|_, _, _| {
        Ok(Expr::from(InfiniteConstant::NotANumber))
      })
    )
    .set_derivative(
      builder::arity_two_deriv("log", |arg, base, engine| {
        // Convert to ln(a) / ln(b) and do the Quotient Rule.
        let equivalent_expr = Expr::call("/", vec![
          Expr::call("ln", vec![arg]),
          Expr::call("ln", vec![base]),
        ]);
        engine.differentiate(equivalent_expr)
      })
    )
    .build()
}

pub fn exponent() -> Function {
  // TODO Better results when we have polar complex numbers (see Issue #14)
  FunctionBuilder::new("exp")
    .add_case(
      // Real number case
      builder::arity_one().of_type(expr_to_number()).and_then(|arg, _| {
        let e = Number::from(consts::E);
        let power = pow_real(e, arg);
        Ok(Expr::from(power))
      })
    )
    .add_case(
      // Complex number case
      builder::arity_one().of_type(ExprToComplex).and_then(|arg, _| {
        let e = ComplexNumber::from_real(Number::from(consts::E));
        let power = pow_complex(e, arg.into());
        Ok(Expr::from(power))
      })
    )
    .add_case(
      // Interval case
      builder::arity_one().of_type(prisms::expr_to_interval()).and_then(|arg, _| {
        let arg = Interval::from(arg);
        let value = arg.map_monotone(|x| {
          let e = Number::from(consts::E);
          // unwrap: Raising a positive constant to a power will always get a real result.
          pow_real(e, x).unwrap_real()
        });
        Ok(Expr::from(value))
      })
    )
    .add_case(
      // Infinity case
      builder::arity_one().of_type(prisms::ExprToInfinity).and_then(|arg, _| {
        match arg {
          InfiniteConstant::PosInfinity => Ok(Expr::from(InfiniteConstant::PosInfinity)),
          InfiniteConstant::NegInfinity => Ok(Expr::zero()),
          InfiniteConstant::UndirInfinity | InfiniteConstant::NotANumber => Ok(Expr::from(InfiniteConstant::NotANumber)),
        }
      })
    )
    .set_derivative(
      builder::arity_one_deriv("exp", |arg, engine| {
        let arg_deriv = engine.differentiate(arg.clone())?;
        Ok(Expr::call("*", vec![
          arg_deriv,
          Expr::call("exp", vec![arg]),
        ]))
      })
    )
    .build()
}
