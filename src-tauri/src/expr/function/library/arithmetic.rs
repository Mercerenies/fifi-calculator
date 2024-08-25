
//! Basic arithmetic function evaluation rules.

use crate::expr::Expr;
use crate::expr::interval::{Interval, interval_div, interval_div_inexact,
                            interval_recip, interval_recip_inexact, includes_infinity};
use crate::expr::function::{Function, FunctionContext};
use crate::expr::function::table::FunctionTable;
use crate::expr::function::builder::{self, FunctionBuilder, FunctionCaseResult};
use crate::expr::vector::Vector;
use crate::expr::vector::matrix::Matrix;
use crate::expr::vector::tensor::Tensor;
use crate::expr::prisms::{self, expr_to_number, ExprToComplex, ExprToQuaternion};
use crate::expr::predicates;
use crate::expr::number::{Number, ComplexNumber, Quaternion, QuaternionLike,
                          pow_real, pow_complex, pow_complex_to_real};
use crate::expr::number::inexact::{DivInexact, WithInexactDiv};
use crate::expr::simplifier::error::{SimplifierError, DomainError};
use crate::expr::calculus::DifferentiationError;
use crate::expr::algebra::infinity::{InfiniteConstant, UnboundedNumber, is_infinite_constant,
                                     multiply_infinities, infinite_pow};
use crate::graphics::GRAPHICS_NAME;
use crate::util::{repeated, TryPow};
use crate::util::prism::Identity;
use crate::util::matrix::{Matrix as UtilMatrix, SingularMatrixError};

use num::{Zero, One, BigInt};
use either::Either;
use try_traits::ops::{TryAdd, TrySub, TryMul, TryDiv};

use std::cmp::Ordering;

pub fn append_arithmetic_functions(table: &mut FunctionTable) {
  table.insert(addition());
  table.insert(subtraction());
  table.insert(multiplication());
  table.insert(division());
  table.insert(power());
  table.insert(modulo());
  table.insert(floor_division());
  table.insert(arithmetic_negate());
  table.insert(reciprocal());
  table.insert(abs());
  table.insert(signum());
}

pub fn addition() -> Function {
  FunctionBuilder::new("+")
    .permit_flattening()
    .permit_reordering()
    .set_identity(Expr::is_zero)
    .add_partial_eval_rule(Box::new(predicates::is_quaternion))
    .add_partial_eval_rule(Box::new(predicates::is_tensor))
    .add_partial_eval_rule(Box::new(predicates::is_string))
    .add_partial_eval_rule(Box::new(predicates::is_complex_or_inf))
    .add_partial_eval_rule(Box::new(predicates::is_unbounded_interval_like))
    .add_case(
      // Unary simplification
      builder::arity_one().and_then(|arg, _| {
        Ok(arg)
      })
    )
    .add_case(
      // Real number addition
      builder::any_arity().of_type(expr_to_number()).and_then(|args, _| {
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
    .add_case(
      // Quaternion addition
      builder::any_arity().of_type(ExprToQuaternion).and_then(|args, _| {
        let sum = args.into_iter()
          .map(Quaternion::from)
          .reduce(|a, b| a + b)
          .unwrap_or(Quaternion::zero());
        Ok(Expr::from(sum))
      })
    )
    .add_case(
      // Vector addition (with broadcasting)
      builder::any_arity().of_type(prisms::ExprToTensor).and_then(|args, context| {
        if let Err(err) = Tensor::check_compatible_lengths(args.iter()) {
          context.errors.push(SimplifierError::new("+", err));
          return Err(args);
        }
        // unwrap: Now that we've validated the lengths, we can safely
        // add all of the values together.
        let sum = args.into_iter()
          .fold(Tensor::zero(), |a, b| a.try_add(b).unwrap());
        Ok(sum.into())
      })
    )
    .add_case(
      // String concatenation
      builder::any_arity().of_type(prisms::expr_to_string()).and_then(|args, _| {
        Ok(Expr::from(args.join("")))
      })
    )
    .add_case(
      // Infinity addition
      builder::any_arity().of_type(prisms::expr_to_complex_or_inf()).and_then(|args, _| {
        // We can ignore any finite quantities, since they hold no
        // sway over the result.
        let sum = args.into_iter()
          .filter_map(Either::right)
          .reduce(|a, b| a + b)
          .unwrap(); // unwrap safety: One of the earlier cases would have triggered if there were no infinities.
        Ok(Expr::from(sum))
      })
    )
    .add_case(
      // Interval addition
      builder::any_arity().of_type(prisms::expr_to_unbounded_interval_like()).and_then(|args, ctx| {
        let sum = args.into_iter()
          .map(Interval::from)
          .try_fold(Interval::singleton(UnboundedNumber::zero()), |a, b| a.try_add(b));
        match sum {
          Ok(sum) => Ok(Expr::from(sum)),
          Err(err) => {
            ctx.errors.push(SimplifierError::new("+", err));
            Ok(Expr::from(InfiniteConstant::NotANumber))
          }
        }
      })
    )
    .add_case(
      // Graphics object concatenation
      builder::any_arity().of_type(prisms::Graphics2D::prism()).and_then(|args, _| {
        let args = args.into_iter()
          .flat_map(prisms::Graphics2D::into_args)
          .collect();
        Ok(Expr::call(GRAPHICS_NAME, args))
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
      builder::arity_two().both_of_type(expr_to_number()).and_then(|arg1, arg2, _| {
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
    .add_case(
      // Quaternion subtraction
      builder::arity_two().both_of_type(ExprToQuaternion).and_then(|arg1, arg2, _| {
        let difference = Quaternion::from(arg1) - Quaternion::from(arg2);
        Ok(Expr::from(difference))
      })
    )
    .add_case(
      // Vector subtraction (with broadcasting)
      builder::arity_two().both_of_type(prisms::ExprToTensor).and_then(|arg1, arg2, context| {
        if let Err(err) = Tensor::check_compatible_lengths([&arg1, &arg2]) {
          context.errors.push(SimplifierError::new("-", err));
          return Err((arg1, arg2));
        }
        // unwrap: Now that we've validated the lengths, we can safely
        // subtract.
        Ok(Expr::from(arg1.try_sub(arg2).unwrap()))
      })
    )
    .add_case(
      // Pure infinity subtraction
      builder::arity_two().both_of_type(prisms::ExprToInfinity).and_then(|arg1, arg2, _| {
        Ok(Expr::from(arg1 - arg2))
      })
    )
    .add_case(
      // Infinity minus scalar
      builder::arity_two().of_types(prisms::ExprToInfinity, ExprToComplex).and_then(|arg1, _arg2, _| {
        Ok(Expr::from(arg1))
      })
    )
    .add_case(
      // Scalar minus infinity
      builder::arity_two().of_types(ExprToComplex, prisms::ExprToInfinity).and_then(|_arg1, arg2, _| {
        Ok(Expr::from(- arg2))
      })
    )
    .add_case(
      // Interval subtraction
      builder::arity_two().both_of_type(prisms::expr_to_unbounded_interval_like()).and_then(|arg1, arg2, ctx| {
        match Interval::from(arg1).try_sub(Interval::from(arg2)) {
          Ok(interval) => Ok(Expr::from(interval)),
          Err(err) => {
            ctx.errors.push(SimplifierError::new("-", err));
            Ok(Expr::from(InfiniteConstant::NotANumber))
          }
        }
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
    .permit_reordering()
    .set_identity(Expr::is_one)
    .add_partial_eval_rule(Box::new(predicates::is_tensor))
    .add_partial_eval_rule(Box::new(predicates::is_complex_or_inf))
    .add_partial_eval_rule(Box::new(predicates::is_unbounded_interval_like))
    .add_case(
      // Unary simplification
      builder::arity_one().and_then(|arg, _| {
        Ok(arg)
      })
    )
    .add_case(
      // String repetition
      builder::arity_two().of_types(prisms::expr_to_string(), prisms::expr_to_usize()).and_then(|s, n, _| {
        let repeated_str: String = repeated(s, n);
        Ok(Expr::from(repeated_str))
      })
    )
    .add_case(
      // Multiplication by zero
      Box::new(|args, _context| {
        // TODO: The manual construction of FunctionCase and explicit
        // FunctionCaseResults here are less than ideal. Can we make a
        // builder for this?
        let contains_zero = args.iter().any(Expr::is_zero);
        let contains_infinities = args.iter().any(is_infinite_constant);
        if contains_zero && !contains_infinities {
          FunctionCaseResult::Success(Expr::zero())
        } else {
          FunctionCaseResult::NoMatch(args)
        }
      })
    )
    .add_case(
      // Real number multiplication
      builder::any_arity().of_type(expr_to_number()).and_then(|args, _| {
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
    .add_case(
      // Quaternions (Trap case; you must use '@' not '*' for
      // non-commutative multiplication)
      builder::any_arity().of_type(ExprToQuaternion).and_then(|args, context| {
        context.errors.push(
          SimplifierError::new("*", DomainError::new("Quaternion multiplication is non-commutative; use @ not *")),
        );
        Err(args)
      })
    )
    .add_case(
      // Vector multiplication (with broadcasting)
      builder::any_arity().of_type(prisms::ExprToTensor).and_then(|args, context| {
        if let Err(err) = Tensor::check_compatible_lengths(args.iter()) {
          context.errors.push(SimplifierError::new("*", err));
          return Err(args);
        }
        // unwrap: Now that we've validated the lengths, we can safely
        // multiply all of the values together.
        let product = args.into_iter()
          .fold(Tensor::one(), |a, b| a.try_mul(b).unwrap());
        Ok(product.into())
      })
    )
    .add_case(
      // Infinity multiplication
      builder::any_arity().of_type(prisms::expr_to_complex_or_inf()).and_then(|args, _| {
        Ok(multiply_infinities(args))
      })
    )
    .add_case(
      // Interval multiplication
      builder::any_arity().of_type(prisms::expr_to_unbounded_interval_like()).and_then(|args, ctx| {
        let product = args.into_iter()
          .map(Interval::from)
          .try_fold(Interval::singleton(UnboundedNumber::one()), |a, b| a.try_mul(b));
        match product {
          Ok(product) => Ok(Expr::from(product)),
          Err(err) => {
            ctx.errors.push(SimplifierError::new("*", err));
            Ok(Expr::from(InfiniteConstant::NotANumber))
          }
        }
      })
    )
    .set_derivative(
      |args, engine| {
        let mut final_terms = Vec::with_capacity(args.len());
        for i in 0..args.len() {
          let mut args = args.clone();
          args[i].mutate_failable(|e| engine.differentiate(e))?;

          // To help the simplifier along, if args[i]'s derivative is
          // zero, then omit the term entirely.
          if args[i].is_zero() {
            continue;
          }

          final_terms.push(Expr::call("*", args));
        }
        Ok(Expr::call("+", final_terms))
      }
    )
    .build()
}

pub fn division() -> Function {
  FunctionBuilder::new("/")
    .add_case(
      // Division by one
      builder::arity_two().of_types(Identity, prisms::ExprToOne).and_then(|arg1, _, _| {
        Ok(arg1)
      })
    )
    .add_case(
      // Real number division
      builder::arity_two().both_of_type(expr_to_number()).and_then(|arg1, arg2, context| {
        if arg2.is_zero() {
          return division_by_zero(context, "/", (arg1, arg2));
        }
        let quotient = if context.calculation_mode.has_fractional_flag() {
          arg1 / arg2
        } else {
          arg1.div_inexact(&arg2)
        };
        Ok(Expr::from(quotient))
      })
    )
    .add_case(
      // Complex number division
      builder::arity_two().both_of_type(ExprToComplex).and_then(|arg1, arg2, context| {
        if arg2.is_zero() {
          return division_by_zero(context, "/", (arg1, arg2));
        }
        let arg1 = ComplexNumber::from(arg1);
        let arg2 = ComplexNumber::from(arg2);
        let quotient = if context.calculation_mode.has_fractional_flag() {
          arg1 / arg2
        } else {
          arg1.div_inexact(&arg2)
        };
        Ok(Expr::from(quotient))
      })
    )
    .add_case(
      // Vector division (with broadcasting)
      builder::arity_two().both_of_type(prisms::ExprToTensor).and_then(|arg1, arg2, context| {
        if let Err(err) = Tensor::check_compatible_lengths([&arg1, &arg2]) {
          context.errors.push(SimplifierError::new("/", err));
          return Err((arg1, arg2));
        }
        // safety: We checked that the lengths were compatible, so
        // `Tensor` won't panic. Further, at least one of `arg1` or
        // `arg2` is a vector (since the above cases would have
        // handled any situations where both are scalar), so the
        // division operator here merely simplifies the expression and
        // does NOT invoke Number::div, ComplexNumber::div, or
        // Quaternion::div, so division by zero will NOT panic here.
        Ok(Expr::from(arg1.try_div(arg2).unwrap()))
      })
    )
    .add_case(
      // Infinity division (infinity divided by complex)
      builder::arity_two().of_types(prisms::ExprToInfinity, ExprToComplex).and_then(|arg1, arg2, _| {
        if arg1 == InfiniteConstant::NotANumber {
          Ok(Expr::from(InfiniteConstant::NotANumber))
        } else if arg2.is_zero() {
          Ok(Expr::from(InfiniteConstant::UndirInfinity))
        } else {
          Ok(multiply_infinities(vec![
            Either::Right(arg1),
            Either::Left(arg2.recip()),
          ]))
        }
      })
    )
    .add_case(
      // Infinity division (Complex divided by infinity)
      builder::arity_two().of_types(ExprToComplex, prisms::ExprToInfinity).and_then(|_arg1, arg2, _| {
        if arg2 == InfiniteConstant::NotANumber {
          Ok(Expr::from(InfiniteConstant::NotANumber))
        } else {
          Ok(Expr::zero())
        }
      })
    )
    .add_case(
      // Infinity division
      builder::arity_two().both_of_type(prisms::ExprToInfinity).and_then(|_, _, _| {
        // Cannot divide two infinities; the quantity of the result is
        // not known, so produce NaN.
        Ok(Expr::from(InfiniteConstant::NotANumber))
      })
    )
    .add_case(
      // Interval division
      builder::arity_two().both_of_type(prisms::expr_to_unbounded_interval_like()).and_then(|arg1, arg2, ctx| {
        let arg1_interval = Interval::from(arg1.clone());
        let arg2_interval = Interval::from(arg2.clone());

        let inputs_have_infinity = includes_infinity(&arg1_interval) || includes_infinity(&arg2_interval);

        let quotient = if ctx.calculation_mode.has_fractional_flag() {
          interval_div(arg1_interval, arg2_interval)
        } else {
          interval_div_inexact(arg1_interval, arg2_interval)
        };
        match quotient {
          Err(err) => {
            ctx.errors.push(SimplifierError::new("/", err));
            Err((arg1, arg2))
          }
          Ok(quotient) => {
            if !inputs_have_infinity && includes_infinity(&quotient) && !ctx.calculation_mode.has_infinity_flag() {
              // Do not produce infinities if the calculation mode
              // doesn't allow it.
              ctx.errors.push(SimplifierError::division_by_zero("/"));
              Err((arg1, arg2))
            } else {
              Ok(quotient.into())
            }
          }
        }
      })
    )
    .set_derivative(
      builder::arity_two_deriv("/", |arg1, arg2, engine| {
        let arg1_deriv = engine.differentiate(arg1.clone())?;
        let arg2_deriv = engine.differentiate(arg2.clone())?;

        if arg2_deriv.is_zero() {
          // Simple case; denominator was a constant
          Ok(Expr::call("/", vec![arg1_deriv, arg2]))
        } else {
          // General Quotient Rule
          let final_numerator = Expr::call("-", vec![
            Expr::call("*", vec![arg1_deriv, arg2.clone()]),
            Expr::call("*", vec![arg1, arg2_deriv]),
          ]);
          let final_denominator = Expr::call("^", vec![arg2, Expr::from(Number::from(2))]);
          Ok(Expr::call("/", vec![final_numerator, final_denominator]))
        }
      })
    )
    .build() // TODO Trap case for quaternions if we implement a division variant of the infix `@`
}

pub fn power() -> Function {
  FunctionBuilder::new("^")
    .add_case(
      // Real number power function
      builder::arity_two().both_of_type(expr_to_number()).and_then(|arg1, arg2, context| {
        if arg1.is_zero() && arg2.is_zero() {
          context.errors.push(SimplifierError::zero_to_zero_power("^"));
          return Err((arg1, arg2));
        }
        if arg1.is_zero() && arg2 < Number::zero() {
          return division_by_zero(context, "^", (arg1, arg2));
        }
        let has_input_ratios = arg1.is_proper_ratio() || arg2.is_proper_ratio();
        let power = pow_real(arg1, arg2);
        let power = if context.calculation_mode.has_fractional_flag() {
          power
        } else if power.has_proper_ratio() && !has_input_ratios {
          power.to_inexact()
        } else {
          power
        };
        Ok(Expr::from(power))
      })
    )
    .add_case(
      // Complex-to-real number power function
      builder::arity_two().of_types(ExprToComplex, expr_to_number()).and_then(|arg1, arg2, context| {
        if arg1.is_zero() && arg2.is_zero() {
          context.errors.push(SimplifierError::zero_to_zero_power("^"));
          return Err((arg1, arg2));
        }
        if arg1.is_zero() && arg2 < Number::zero() {
          return division_by_zero(context, "^", (arg1, arg2));
        }
        let has_input_ratios = arg1.has_proper_ratio() || arg2.is_proper_ratio();
        let power = pow_complex_to_real(arg1.into(), arg2);
        let power = if context.calculation_mode.has_fractional_flag() {
          power
        } else if power.has_proper_ratio() && !has_input_ratios {
          power.to_inexact()
        } else {
          power
        };
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
        let has_input_ratios = arg1.has_proper_ratio() || arg2.has_proper_ratio();
        let power = pow_complex(arg1.into(), arg2.into());
        let power = if context.calculation_mode.has_fractional_flag() {
          power
        } else if power.has_proper_ratio() && !has_input_ratios {
          power.to_inexact()
        } else {
          power
        };
        Ok(Expr::from(power))
      })
    )
    .add_case(
      // Quaternion power function (only integer powers are supported)
      builder::arity_two().of_types(ExprToQuaternion, ExprToQuaternion).and_then(|arg1, arg2, context| {
        let arg2 = match try_quat_into_bigint(arg2) {
          Ok(arg2) => arg2,
          Err(original_arg2) => {
            context.errors.push(SimplifierError::custom_error("^", "Only integer powers are supported for quaternion exponentiation"));
            return Err((arg1, original_arg2));
          }
        };
        let has_input_ratio = arg1.has_proper_ratio(); // Note: arg2 is BigInt and thus never rational.
        let result = Quaternion::from(arg1).powi(arg2);
        let result = if context.calculation_mode.has_fractional_flag() {
          result
        } else if result.has_proper_ratio() && !has_input_ratio {
          result.to_inexact()
        } else {
          result
        };
        Ok(result.into())
      })
    )
    .add_case(
      // Vector to scalar power function
      //
      // (We just distribute scalar exponents over the elements of the
      // vector)
      builder::arity_two().of_types(prisms::ExprToVector, ExprToComplex).and_then(|arg1, arg2, _| {
        let result = arg1.map(|elem| Expr::call("^", vec![elem, Expr::from(arg2.clone())]));
        Ok(result.into_expr())
      })
    )
    .add_case(
      // Interval to scalar power function
      builder::arity_two().of_types(prisms::expr_to_unbounded_interval(), prisms::expr_to_i64()).and_then(|arg1, arg2, ctx| {
        let arg1_interval = Interval::from(arg1.clone());
        let input_has_infinity = includes_infinity(&arg1_interval);
        let result = if arg2 < 0 {
          let recip = if ctx.calculation_mode.has_fractional_flag() {
            interval_recip(arg1_interval)
          } else {
            interval_recip_inexact(arg1_interval)
          };
          recip.try_pow(- arg2)
        } else {
          arg1_interval.try_pow(arg2)
        };
        match result {
          Err(err) => {
            ctx.errors.push(SimplifierError::new("^", err));
            Err((arg1, arg2))
          }
          Ok(result) => {
            if !input_has_infinity && includes_infinity(&result) && !ctx.calculation_mode.has_infinity_flag() {
              // Do not produce infinities if the calculation mode
              // doesn't allow it.
              ctx.errors.push(SimplifierError::division_by_zero("^"));
              Err((arg1, arg2))
            } else {
              Ok(result.into())
            }
          }
        }
      })
    )
    .add_case(
      // Infinity to real power
      builder::arity_two().of_types(prisms::ExprToInfinity, expr_to_number()).and_then(|arg1, arg2, _| {
        match arg2.cmp(&Number::zero()) {
          Ordering::Greater => {
            if arg1 == InfiniteConstant::NegInfinity {
              // Rotate the scalar part around the complex plane.
              // (TODO: Better results if arg2 is a positive integer?)
              Ok(Expr::call("*", vec![
                Expr::from(pow_complex_to_real(ComplexNumber::from_real(-1), arg2)),
                Expr::from(InfiniteConstant::PosInfinity),
              ]))
            } else {
              Ok(arg1.into())
            }
          }
          Ordering::Equal => {
            // Infinity^0 is always NaN
            Ok(Expr::from(InfiniteConstant::NotANumber))
          }
          Ordering::Less => {
            // Infinity^(-n) is zero, NaN^(-n) is NaN
            if arg1 == InfiniteConstant::NotANumber {
              Ok(Expr::from(InfiniteConstant::NotANumber))
            } else {
              Ok(Expr::zero())
            }
          }
        }
      })
    )
    .add_case(
      // Infinity to complex power (not supported, always NaN)
      builder::arity_two().of_types(prisms::ExprToInfinity, ExprToComplex).and_then(|_, _, _| {
        Ok(Expr::from(InfiniteConstant::NotANumber))
      })
    )
    .add_case(
      // Infinity to infinite power
      builder::arity_two().of_types(prisms::ExprToInfinity, prisms::ExprToInfinity).and_then(|arg1, arg2, _| {
        Ok(infinite_pow(arg1, arg2))
      })
    )
    .add_case(
      // Positive real to infinity power
      builder::arity_two().of_types(prisms::expr_to_positive_number(), prisms::ExprToInfinity).and_then(|arg1, arg2, _| {
        let arg1 = Number::from(arg1);
        match arg2 {
          InfiniteConstant::PosInfinity => {
            match arg1.cmp(&Number::one()) {
              Ordering::Greater => Ok(Expr::from(InfiniteConstant::PosInfinity)),
              Ordering::Less => Ok(Expr::zero()),
              Ordering::Equal => Ok(Expr::one()),
            }
          }
          InfiniteConstant::NegInfinity => {
            match arg1.cmp(&Number::one()) {
              Ordering::Greater => Ok(Expr::zero()),
              Ordering::Less => Ok(Expr::from(InfiniteConstant::PosInfinity)),
              Ordering::Equal => Ok(Expr::one())
            }
          }
          InfiniteConstant::UndirInfinity | InfiniteConstant::NotANumber => {
            Ok(Expr::from(InfiniteConstant::NotANumber))
          }
        }
      })
    )
    .add_case(
      // Complex to infinity power
      builder::arity_two().of_types(ExprToComplex, prisms::ExprToInfinity).and_then(|arg1, arg2, _| {
        match arg2 {
          InfiniteConstant::PosInfinity => {
            match arg1.abs().cmp(&Number::one()) {
              Ordering::Greater => Ok(Expr::from(InfiniteConstant::PosInfinity)),
              Ordering::Less => Ok(Expr::zero()),
              Ordering::Equal => Ok(Expr::from(InfiniteConstant::NotANumber)),
            }
          }
          InfiniteConstant::NegInfinity => {
            match arg1.abs().cmp(&Number::one()) {
              Ordering::Greater => Ok(Expr::zero()),
              Ordering::Less => Ok(Expr::from(InfiniteConstant::PosInfinity)),
              Ordering::Equal => Ok(Expr::from(InfiniteConstant::NotANumber)),
            }
          }
          InfiniteConstant::UndirInfinity | InfiniteConstant::NotANumber => {
            Ok(Expr::from(InfiniteConstant::NotANumber))
          }
        }
      })
    )
    .set_derivative(
      builder::arity_two_deriv("^", |arg1, arg2, engine| {
        // TODO: Write a variant of postorder_walk that produces `()`
        // and doesn't consume its input, so we don't have to clone()
        // here to get free_vars().
        let arg1_has_var = arg1.clone().free_vars().contains(engine.target_variable());
        let arg2_has_var = arg2.clone().free_vars().contains(engine.target_variable());

        if !arg1_has_var && !arg2_has_var {
          // Constant
          Ok(Expr::zero())
        } else if !arg2_has_var {
          // Power Rule
          let arg1_deriv = engine.differentiate(arg1.clone())?;
          Ok(Expr::call("*", vec![
            arg2.clone(),
            arg1_deriv,
            Expr::call("^", vec![arg1, Expr::call("-", vec![arg2, Expr::from(1)])]),
          ]))
        } else if !arg1_has_var {
          // Exponent Rule
          let arg2_deriv = engine.differentiate(arg2.clone())?;
          Ok(Expr::call("*", vec![
            Expr::call("ln", vec![arg1.clone()]),
            arg2_deriv,
            Expr::call("^", vec![arg1, arg2]),
          ]))
        } else {
          // General Case
          //
          // Technically, this rule always works, but to make it
          // easier on the simplifier, we use more basic rules when we
          // can.
          let arg1_deriv = engine.differentiate(arg1.clone())?;
          let arg2_deriv = engine.differentiate(arg2.clone())?;
          Ok(
            Expr::call("*", vec![
              Expr::call("^", vec![
                arg1.clone(),
                Expr::call("-", vec![arg2.clone(), Expr::from(1)]),
              ]),
              Expr::call("+", vec![
                Expr::call("*", vec![arg2.clone(), arg1_deriv]),
                Expr::call("*", vec![
                  arg1.clone(),
                  arg2_deriv,
                  Expr::call("ln", vec![arg1]),
                ]),
              ]),
            ]),
          )
        }
      })
    )
    .build()
}

fn try_quat_into_bigint(quat: QuaternionLike) -> Result<BigInt, QuaternionLike> {
  match quat {
    QuaternionLike::Real(r) => {
      BigInt::try_from(r).map_err(|err| QuaternionLike::Real(err.number))
    }
    _ => Err(quat),
  }
}

pub fn modulo() -> Function {
  FunctionBuilder::new("%")
    .add_case(
      // Real modulo
      builder::arity_two().both_of_type(expr_to_number()).and_then(|arg1, arg2, context| {
        if arg2.is_zero() {
          context.errors.push(SimplifierError::division_by_zero("%"));
          return Err((arg1, arg2));
        }
        Ok(Expr::from(arg1 % arg2))
      })
    )
    .add_case(
      // Trap case: Complex numbers or quaternions
      builder::arity_two().both_of_type(ExprToQuaternion).and_then(|arg1, arg2, context| {
        context.errors.push(SimplifierError::expected_real("%"));
        Err((arg1, arg2))
      })
    )
    .set_derivative(
      builder::arity_two_deriv("%", |arg1, arg2, engine| {
        // Currently, we only support differentiating the modulo
        // operator if the right-hand argument is constant. In that
        // case, the derivative is simply the derivative of the
        // left-hand argument, except on a countable subset of the
        // reals, on which the derivative fails to exist. The function
        // *is* differentiable in the general case, but I haven't
        // worked out how to do it.
        let free_vars = arg2.free_vars();
        if free_vars.contains(engine.target_variable()) {
          let err = DifferentiationError::custom_error("Right-hand side of % must be constant to differentiate");
          return Err(engine.error(err));
        }
        engine.differentiate(arg1)
      })
    )
    .build()
}

pub fn floor_division() -> Function {
  FunctionBuilder::new("div")
    .add_case(
      // Real floor div
      builder::arity_two().both_of_type(expr_to_number()).and_then(|arg1, arg2, context| {
        if arg2.is_zero() {
          return division_by_zero_or_nan(context, "div", (arg1, arg2));
        }
        Ok(Expr::from(arg1.div_floor(&arg2)))
      })
    )
    .add_case(
      // Trap case: Complex numbers / quaternions
      builder::arity_two().both_of_type(ExprToQuaternion).and_then(|arg1, arg2, context| {
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
      builder::arity_one().of_type(expr_to_number()).and_then(|arg, _| {
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
    .add_case(
      // Quaternion negation
      builder::arity_one().of_type(ExprToQuaternion).and_then(|arg, _| {
        let arg = Quaternion::from(arg);
        Ok(Expr::from(- arg))
      })
    )
    .add_case(
      // Negation of a vector
      builder::arity_one().of_type(prisms::ExprToVector).and_then(|arg, _| {
        let result = arg.map(|e| Expr::call("negate", vec![e]));
        Ok(result.into_expr())
      })
    )
    .add_case(
      // Negation of an interval
      builder::arity_one().of_type(prisms::expr_to_unbounded_interval()).and_then(|arg, _| {
        let arg = Interval::from(arg);
        Ok(Expr::from(- arg))
      })
    )
    .add_case(
      // Negation of infinity
      builder::arity_one().of_type(prisms::ExprToInfinity).and_then(|arg, _| {
        Ok(Expr::from(- arg))
      })
    )
    .set_derivative(
      builder::arity_one_deriv("negate", |arg, engine| {
        Ok(Expr::call("negate", vec![engine.differentiate(arg)?]))
      })
    )
    .build()
}

pub fn reciprocal() -> Function {
  FunctionBuilder::new("recip")
    .add_case(
      // Real / Complex number reciprocal
      builder::arity_one().of_type(ExprToComplex).and_then(|arg, ctx| {
        if arg.is_zero() {
          return division_by_zero(ctx, "recip", arg);
        }
        let result = if ctx.calculation_mode.has_fractional_flag() {
          arg.map(|r| r.recip(), |c| c.recip())
        } else {
          arg.map(|r| r.recip_inexact(), |c| c.recip_inexact())
        };
        Ok(Expr::from(result))
      })
    )
    .add_case(
      // Quaternion reciprocal
      builder::arity_one().of_type(ExprToQuaternion).and_then(|arg, ctx| {
        if arg.is_zero() {
          return division_by_zero(ctx, "recip", arg);
        }
        let arg = Quaternion::from(arg);
        if ctx.calculation_mode.has_fractional_flag() {
          Ok(Expr::from(arg.recip()))
        } else {
          Ok(Expr::from(arg.recip_inexact()))
        }
      })
    )
    .add_case(
      // Inverse of a matrix
      builder::arity_one().of_type(prisms::ExprToTypedMatrix::new(ExprToComplex)).and_then(|mat, ctx| {
        if mat.width() != mat.height() {
          ctx.errors.push(SimplifierError::custom_error("recip", "Expected square matrix"));
          return Err(mat);
        }
        let original_mat = mat.clone();
        let mat = mat.map(ComplexNumber::from);
        match inverse_matrix(mat, ctx) {
          Ok(mat) => Ok(Expr::from(Matrix::from(mat.map(Expr::from)))),
          Err(err) => {
            ctx.errors.push(SimplifierError::new("recip", err));
            Err(original_mat)
          }
        }
      })
    )
    .add_case(
      // Vector reciprocal (trap case; you probably intended a matrix)
      builder::arity_one().of_type(prisms::ExprToVector).and_then(|arg, ctx| {
        ctx.errors.push(SimplifierError::custom_error("recip", "Expected matrix"));
        Err(arg)
      })
    )
    .add_case(
      // Reciprocal of an interval
      builder::arity_one().of_type(prisms::expr_to_unbounded_interval()).and_then(|arg, _| {
        Ok(Expr::call("/", vec![Expr::from(1), Expr::from(arg)]))
      })
    )
    .add_case(
      // Reciprocal of infinity
      builder::arity_one().of_type(prisms::ExprToInfinity).and_then(|arg, _| {
        match arg {
          InfiniteConstant::PosInfinity | InfiniteConstant::NegInfinity => Ok(Expr::zero()),
          other_inf => Ok(other_inf.into())
        }
      })
    )
    .set_derivative(
      builder::arity_one_deriv("recip", |arg, engine| {
        let arg_deriv = engine.differentiate(arg.clone())?;
        Ok(Expr::call("/", vec![
          Expr::call("*", vec![Expr::from(-1), arg_deriv]),
          Expr::call("^", vec![arg, Expr::from(2)]),
        ]))
      })
    )
    .build()
}

fn inverse_matrix(
  mat: UtilMatrix<ComplexNumber>,
  ctx: &FunctionContext,
) -> Result<UtilMatrix<ComplexNumber>, SingularMatrixError> {
  if ctx.calculation_mode.has_fractional_flag() {
    mat.inverse_matrix()
  } else {
    let res = mat.map(WithInexactDiv).inverse_matrix()?;
    Ok(res.map(|x| x.0))
  }
}

pub fn abs() -> Function {
  FunctionBuilder::new("abs")
    .add_case(
      // Real number abs
      builder::arity_one().of_type(expr_to_number()).and_then(|arg, _| {
        Ok(Expr::from(arg.abs()))
      })
    )
    .add_case(
      // Complex number abs
      builder::arity_one().of_type(ExprToComplex).and_then(|arg, _| {
        let arg = ComplexNumber::from(arg);
        // TODO If and when we make complex abs() exact, this will
        // need to be calibrated for the calculation mode's fractional
        // flag.
        Ok(Expr::from(arg.abs()))
      })
    )
    .add_case(
      // Quaternion length
      builder::arity_one().of_type(ExprToQuaternion).and_then(|arg, _| {
        let arg = Quaternion::from(arg);
        // TODO If and when we make quaternion abs() exact, this will
        // need to be calibrated for the calculation mode's fractional
        // flag.
        Ok(Expr::from(arg.abs()))
      })
    )
    .add_case(
      // Norm of a vector
      builder::arity_one().of_type(prisms::ExprToVector).and_then(|arg, _| {
        Ok(vector_norm(arg))
      })
    )
    .add_case(
      // Absolute value of infinity
      builder::arity_one().of_type(prisms::ExprToInfinity).and_then(|arg, _| {
        Ok(Expr::from(arg.abs()))
      })
    )
    .set_derivative(
      builder::arity_one_deriv("abs", |arg, engine| {
        Ok(Expr::call("*", vec![
          Expr::call("signum", vec![arg.clone()]),
          engine.differentiate(arg)?,
        ]))
      })
    )
    .build()
}

pub fn signum() -> Function {
  FunctionBuilder::new("signum")
    .add_case(
      // Real number / complex number / quaternion signum
      builder::arity_one().of_type(ExprToQuaternion).and_then(|arg, _| {
        let arg = arg.map(|r| r.signum(), |z| z.signum(), |q| q.signum());
        Ok(Expr::from(arg))
      })
    )
    .add_case(
      // Normalized vector
      builder::arity_one().of_type(prisms::ExprToVector).and_then(|arg, _| {
        let norm = vector_norm(arg.clone());
        Ok(Expr::call("/", vec![
          arg.into(),
          Expr::call("||", vec![norm, Expr::from(Number::from(1))]), // Corner case: If vector is zero, return zero
        ]))
      })
    )
    .add_case(
      // Signum of infinity
      builder::arity_one().of_type(prisms::ExprToInfinity).and_then(|arg, ctx| {
        match arg {
          InfiniteConstant::PosInfinity => Ok(Expr::from(1)),
          InfiniteConstant::NegInfinity => Ok(Expr::from(-1)),
          arg => {
            ctx.errors.push(SimplifierError::custom_error("signum", "Cannot take signum of unsigned infinity"));
            Err(arg)
          }
        }
      })
    )
    .set_derivative(
      builder::arity_one_deriv("signum", |_, _| {
        Ok(Expr::zero())
      })
    )
    .build()
}

fn vector_norm(vec: Vector) -> Expr {
  let addends = vec.into_iter().map(|x| Expr::call("^", vec![x, Expr::from(2)])).collect();
  Expr::call("^", vec![
    Expr::call("+", addends),
    Expr::from(Number::ratio(1, 2)),
  ])
}

/// Returns [`InfiniteConstant::UndirInfinity`] if the infinity flag
/// on `context` is set, or produces an error and refuses to evaluate
/// otherwise.
fn division_by_zero<E>(context: &mut FunctionContext, function_name: &str, err: E) -> Result<Expr, E> {
  if context.calculation_mode.has_infinity_flag() {
    Ok(Expr::from(InfiniteConstant::UndirInfinity))
  } else {
    context.errors.push(SimplifierError::division_by_zero(function_name));
    Err(err)
  }
}

/// Returns [`InfiniteConstant::NotANumber`] if the infinity flag
/// on `context` is set, or produces an error and refuses to evaluate
/// otherwise.
fn division_by_zero_or_nan<E>(context: &mut FunctionContext, function_name: &str, err: E) -> Result<Expr, E> {
  if context.calculation_mode.has_infinity_flag() {
    Ok(Expr::from(InfiniteConstant::NotANumber))
  } else {
    context.errors.push(SimplifierError::division_by_zero(function_name));
    Err(err)
  }
}
