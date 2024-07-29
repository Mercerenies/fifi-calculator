
//! Evaluation rules for transcendental and trigonometric functions.

use crate::expr::Expr;
use crate::expr::simplifier::error::SimplifierError;
use crate::expr::function::{Function, FunctionContext};
use crate::expr::function::table::FunctionTable;
use crate::expr::function::builder::{self, FunctionBuilder};
use crate::expr::prisms::{self, expr_to_number, ExprToComplex};
use crate::expr::number::{Number, ComplexNumber, ComplexLike, pow_real, pow_complex};
use crate::expr::algebra::infinity::{InfiniteConstant, SignedInfinity, UnboundedNumber};
use crate::expr::interval::{RawInterval, Interval, includes_infinity};

use num::{Zero, One};

use std::f64::consts;

pub fn append_transcendental_functions(table: &mut FunctionTable) {
  table.insert(natural_log());
  table.insert(logarithm());
  table.insert(exponent());
  table.insert(sine());
  table.insert(cosine());
  table.insert(tangent());
  table.insert(secant());
  table.insert(cosecant());
  table.insert(cotangent());
  table.insert(sine_hyper());
  table.insert(cosine_hyper());
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
          if ctx.calculation_mode.has_infinity_flag() {
            return Ok(Expr::from(InfiniteConstant::NegInfinity))
          } else {
            ctx.errors.push(SimplifierError::custom_error("ln", "Expected non-zero complex number"));
            return Err(arg);
          }
        }
        let arg = ComplexNumber::from(arg);
        Ok(Expr::from(arg.ln()))
      })
    )
    .add_case(
      // Natural logarithm of interval of positive reals
      builder::arity_one().of_type(prisms::expr_to_unbounded_interval()).and_then(|arg, ctx| {
        ln_of_interval(arg, ctx).map(Expr::from)
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

fn ln_of_interval(
  arg: RawInterval<UnboundedNumber>,
  ctx: &mut FunctionContext,
) -> Result<Interval<UnboundedNumber>, RawInterval<UnboundedNumber>> {
  if arg.left < UnboundedNumber::zero() || arg.right <= UnboundedNumber::zero() {
    ctx.errors.push(SimplifierError::custom_error("ln", "Expected interval of positive reals"));
    return Err(arg);
  }
  let arg_interval = Interval::from(arg.clone());
  let arg_has_infinity = includes_infinity(&arg_interval);

  let result_interval = arg_interval.map_monotone(ln_unbounded);
  if !arg_has_infinity && includes_infinity(&result_interval) && !ctx.calculation_mode.has_infinity_flag() {
    ctx.errors.push(SimplifierError::custom_error("ln", "Expected interval of positive reals"));
    return Err(arg);
  }

  Ok(result_interval)
}

/// Panics if input < 0.
fn ln_unbounded(input: UnboundedNumber) -> UnboundedNumber {
  assert!(input >= UnboundedNumber::zero());
  if input == UnboundedNumber::zero() {
    return UnboundedNumber::Infinite(SignedInfinity::NegInfinity);
  }
  match input {
    UnboundedNumber::Finite(x) => UnboundedNumber::Finite(x.ln()),
    UnboundedNumber::Infinite(SignedInfinity::PosInfinity) => UnboundedNumber::Infinite(SignedInfinity::PosInfinity),
    UnboundedNumber::Infinite(SignedInfinity::NegInfinity) => unreachable!(),
  }
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
        if base.is_zero() {
          return Ok(Expr::zero());
        }
        if arg.is_zero() {
          if ctx.calculation_mode.has_infinity_flag() {
            // Let `ln` and `/` do the work.
            return Ok(Expr::call("/", vec![
              Expr::call("ln", vec![arg.into()]),
              Expr::call("ln", vec![base.into()]),
            ]));
          } else {
            ctx.errors.push(SimplifierError::custom_error("log", "Expected non-zero complex arguments"));
            return Err((arg, base));
          }
        }
        if base.is_one() {
          if ctx.calculation_mode.has_infinity_flag() {
            return Ok(Expr::from(InfiniteConstant::UndirInfinity));
          } else {
            ctx.errors.push(SimplifierError::division_by_zero("log"));
            return Err((arg, base));
          }
        }
        let arg = ComplexNumber::from(arg);
        let base = ComplexNumber::from(base);
        Ok(Expr::from(arg.ln() / base.ln()))
      })
    )
    .add_case(
      // Arbitrary-base logarithm of an interval with positive real base
      builder::arity_two().of_types(prisms::expr_to_unbounded_interval(), prisms::expr_to_positive_number()).and_then(|arg, base, ctx| {
        if arg.left < UnboundedNumber::zero() || arg.right <= UnboundedNumber::zero() {
          ctx.errors.push(SimplifierError::custom_error("log", "Expected interval of positive reals"));
          return Err((arg, base));
        }
        let result = ln_of_interval(arg, ctx).map_err(|arg| (arg, base.clone()))?;
        let result = result.map_monotone(|x| x / Number::from(base.clone()).ln());
        Ok(result.into())
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
      builder::arity_one().of_type(prisms::expr_to_unbounded_interval()).and_then(|arg, _| {
        let arg = Interval::from(arg);
        let value = arg.map_monotone(|x| {
          match x {
            UnboundedNumber::Infinite(SignedInfinity::PosInfinity) => UnboundedNumber::Infinite(SignedInfinity::PosInfinity),
            UnboundedNumber::Infinite(SignedInfinity::NegInfinity) => UnboundedNumber::zero(),
            UnboundedNumber::Finite(x) => {
              let e = Number::from(consts::E);
              // unwrap: Raising a positive constant to a power will always get a real result.
              UnboundedNumber::Finite(pow_real(e, x).unwrap_real())
            }
          }
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

pub fn sine() -> Function {
  FunctionBuilder::new("sin")
    .add_case(
      // Real number case
      builder::arity_one().of_type(expr_to_number()).and_then(|arg, _| {
        Ok(Expr::from(arg.sin()))
      })
    )
    .add_case(
      // Complex number case
      builder::arity_one().of_type(ExprToComplex).and_then(|arg, _| {
        let arg = ComplexNumber::from(arg);
        Ok(Expr::from(arg.sin()))
      })
    )
    .set_derivative(
      builder::arity_one_deriv("sin", |arg, engine| {
        let arg_deriv = engine.differentiate(arg.clone())?;
        Ok(Expr::call("*", vec![
          arg_deriv,
          Expr::call("cos", vec![arg]),
        ]))
      })
    )
    .build()
}

pub fn cosine() -> Function {
  FunctionBuilder::new("cos")
    .add_case(
      // Real number case
      builder::arity_one().of_type(expr_to_number()).and_then(|arg, _| {
        Ok(Expr::from(arg.cos()))
      })
    )
    .add_case(
      // Complex number case
      builder::arity_one().of_type(ExprToComplex).and_then(|arg, _| {
        let arg = ComplexNumber::from(arg);
        Ok(Expr::from(arg.cos()))
      })
    )
    .set_derivative(
      builder::arity_one_deriv("cos", |arg, engine| {
        let arg_deriv = engine.differentiate(arg.clone())?;
        Ok(Expr::call("*", vec![
          Expr::from(Number::from(-1)),
          arg_deriv,
          Expr::call("sin", vec![arg]),
        ]))
      })
    )
    .build()
}

pub fn tangent() -> Function {
  FunctionBuilder::new("tan")
    .add_case(
      // Real number case
      builder::arity_one().of_type(expr_to_number()).and_then(|arg, ctx| {
        let s = arg.sin();
        let c = arg.cos();
        if c.is_zero() {
          if ctx.calculation_mode.has_infinity_flag() {
            Ok(Expr::from(InfiniteConstant::UndirInfinity))
          } else {
            ctx.errors.push(SimplifierError::division_by_zero("tan"));
            Err(arg)
          }
        } else {
          Ok(Expr::from(s / c))
        }
      })
    )
    .add_case(
      // Complex number case
      builder::arity_one().of_type(ExprToComplex).and_then(|arg, ctx| {
        let arg = ComplexNumber::from(arg);
        let s = arg.sin();
        let c = arg.cos();
        if c.is_zero() {
          if ctx.calculation_mode.has_infinity_flag() {
            Ok(Expr::from(InfiniteConstant::UndirInfinity))
          } else {
            ctx.errors.push(SimplifierError::division_by_zero("tan"));
            // Note: If arg was real, then case 1 would have
            // triggered, so we know arg was properly complex.
            Err(ComplexLike::Complex(arg))
          }
        } else {
          Ok(Expr::from(s / c))
        }
      })
    )
    .set_derivative(
      builder::arity_one_deriv("tan", |arg, engine| {
        let arg_deriv = engine.differentiate(arg.clone())?;
        Ok(Expr::call("*", vec![
          arg_deriv,
          Expr::call("^", vec![
            Expr::call("sec", vec![arg]),
            Expr::from(2),
          ]),
        ]))
      })
    )
    .build()
}

pub fn secant() -> Function {
  FunctionBuilder::new("sec")
    .add_case(
      // Complex number case (simplify to cos)
      builder::arity_one().of_type(ExprToComplex).and_then(|arg, _| {
        Ok(Expr::call("/", vec![
          Expr::from(1),
          Expr::call("cos", vec![arg.into()]),
        ]))
      })
    )
    .set_derivative(
      builder::arity_one_deriv("sec", |arg, engine| {
        let arg_deriv = engine.differentiate(arg.clone())?;
        Ok(Expr::call("*", vec![
          arg_deriv,
          Expr::call("sec", vec![arg.clone()]),
          Expr::call("tan", vec![arg]),
        ]))
      })
    )
    .build()
}

pub fn cosecant() -> Function {
  FunctionBuilder::new("csc")
    .add_case(
      // Complex number case (simplify to sin)
      builder::arity_one().of_type(ExprToComplex).and_then(|arg, _| {
        Ok(Expr::call("/", vec![
          Expr::from(1),
          Expr::call("sin", vec![arg.into()]),
        ]))
      })
    )
    .set_derivative(
      builder::arity_one_deriv("csc", |arg, engine| {
        let arg_deriv = engine.differentiate(arg.clone())?;
        Ok(Expr::call("*", vec![
          Expr::from(-1),
          arg_deriv,
          Expr::call("csc", vec![arg.clone()]),
          Expr::call("cot", vec![arg]),
        ]))
      })
    )
    .build()
}

pub fn cotangent() -> Function {
  FunctionBuilder::new("cot")
    .add_case(
      // Complex number case (simplify to tan)
      builder::arity_one().of_type(ExprToComplex).and_then(|arg, _| {
        Ok(Expr::call("/", vec![
          Expr::from(1),
          Expr::call("tan", vec![arg.into()]),
        ]))
      })
    )
    .set_derivative(
      builder::arity_one_deriv("cot", |arg, engine| {
        let arg_deriv = engine.differentiate(arg.clone())?;
        Ok(Expr::call("*", vec![
          Expr::from(-1),
          arg_deriv,
          Expr::call("^", vec![
            Expr::call("cot", vec![arg]),
            Expr::from(2),
          ]),
        ]))
      })
    )
    .build()
}

pub fn sine_hyper() -> Function {
  FunctionBuilder::new("sinh")
    .add_case(
      // Real / Complex number case
      builder::arity_one().of_type(ExprToComplex).and_then(|arg, _| {
        let result = arg.map(|r| r.sinh(), |z| z.sinh());
        Ok(result.into())
      })
    )
    .set_derivative(
      builder::arity_one_deriv("sinh", |arg, engine| {
        let arg_deriv = engine.differentiate(arg.clone())?;
        Ok(Expr::call("*", vec![
          arg_deriv,
          Expr::call("cosh", vec![arg]),
        ]))
      })
    )
    .build()
}

pub fn cosine_hyper() -> Function {
  FunctionBuilder::new("cosh")
    .add_case(
      // Real / Complex number case
      builder::arity_one().of_type(ExprToComplex).and_then(|arg, _| {
        let result = arg.map(|r| r.cosh(), |z| z.cosh());
        Ok(result.into())
      })
    )
    .set_derivative(
      builder::arity_one_deriv("cosh", |arg, engine| {
        let arg_deriv = engine.differentiate(arg.clone())?;
        Ok(Expr::call("*", vec![
          arg_deriv,
          Expr::call("sinh", vec![arg]),
        ]))
      })
    )
    .build()
}
