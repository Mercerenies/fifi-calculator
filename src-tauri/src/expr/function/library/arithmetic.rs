
//! Basic arithmetic function evaluation rules.

use crate::expr::Expr;
use crate::expr::interval::Interval;
use crate::expr::function::Function;
use crate::expr::function::table::FunctionTable;
use crate::expr::function::builder::{self, FunctionBuilder, FunctionCaseResult};
use crate::expr::vector::Vector;
use crate::expr::vector::tensor::Tensor;
use crate::expr::prisms::{self, expr_to_number, expr_to_string,
                          ExprToComplex, ExprToVector, ExprToTensor,
                          ExprToIntervalLike, expr_to_interval, expr_to_usize};
use crate::expr::number::{Number, ComplexNumber, pow_real, pow_complex, pow_complex_to_real};
use crate::expr::simplifier::error::SimplifierError;
use crate::expr::calculus::DifferentiationError;
use crate::graphics::GRAPHICS_NAME;
use crate::util::repeated;

use num::{Zero, One};

use std::ops::{Add, Mul};

pub fn append_arithmetic_functions(table: &mut FunctionTable) {
  table.insert(addition());
  table.insert(subtraction());
  table.insert(multiplication());
  table.insert(division());
  table.insert(power());
  table.insert(modulo());
  table.insert(floor_division());
  table.insert(arithmetic_negate());
  table.insert(abs());
  table.insert(signum());
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
      // Vector addition (with broadcasting)
      builder::any_arity().of_type(ExprToTensor).and_then(|args, context| {
        if let Err(err) = Tensor::check_compatible_lengths(args.iter()) {
          context.errors.push(SimplifierError::new("+", err));
          return Err(args);
        }
        // Now that we've validated the lengths, we can safely add all
        // of the values together. `Broadcastable::add` can panic, but
        // it won't since we know the lengths are good.
        let sum = args.into_iter()
          .fold(Tensor::zero(), Tensor::add);
        Ok(sum.into())
      })
    )
    .add_case(
      // String concatenation
      builder::any_arity().of_type(expr_to_string()).and_then(|args, _| {
        Ok(Expr::from(args.join("")))
      })
    )
    .add_case(
      // Interval addition
      builder::any_arity().of_type(ExprToIntervalLike).and_then(|args, _| {
        let sum = args.into_iter()
          .map(Interval::from)
          .reduce(|a, b| a + b)
          .unwrap(); // unwrap safety: One of the earlier cases would have triggered if arglist was empty
        Ok(Expr::from(sum))
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
      // Vector subtraction (with broadcasting)
      builder::arity_two().both_of_type(ExprToTensor).and_then(|arg1, arg2, context| {
        if let Err(err) = Tensor::check_compatible_lengths([&arg1, &arg2]) {
          context.errors.push(SimplifierError::new("-", err));
          return Err((arg1, arg2));
        }
        Ok(Expr::from(arg1 - arg2))
      })
    )
    .add_case(
      // Interval subtraction
      builder::arity_two().both_of_type(ExprToIntervalLike).and_then(|arg1, arg2, _| {
        Ok(Expr::from(Interval::from(arg1) - Interval::from(arg2)))
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
      // String repetition
      builder::arity_two().of_types(expr_to_string(), expr_to_usize()).and_then(|s, n, _| {
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
        if contains_zero {
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
      // Vector multiplication (with broadcasting)
      builder::any_arity().of_type(ExprToTensor).and_then(|args, context| {
        if let Err(err) = Tensor::check_compatible_lengths(args.iter()) {
          context.errors.push(SimplifierError::new("*", err));
          return Err(args);
        }
        // Now that we've validated the lengths, we can safely add all
        // of the values together. `Tensor::add` can panic, but
        // it won't since we know the lengths are good.
        let product = args.into_iter()
          .fold(Tensor::one(), Tensor::mul);
        Ok(product.into())
      })
    )
    .add_case(
      // Interval multiplication
      builder::any_arity().of_type(ExprToIntervalLike).and_then(|args, _| {
        let sum = args.into_iter()
          .map(Interval::from)
          .reduce(|a, b| a * b)
          .unwrap(); // unwrap safety: One of the earlier cases would have triggered if arglist was empty
        Ok(Expr::from(sum))
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
      // Real number division
      builder::arity_two().both_of_type(expr_to_number()).and_then(|arg1, arg2, context| {
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
    .add_case(
      // Vector division (with broadcasting)
      builder::arity_two().both_of_type(ExprToTensor).and_then(|arg1, arg2, context| {
        if let Err(err) = Tensor::check_compatible_lengths([&arg1, &arg2]) {
          context.errors.push(SimplifierError::new("/", err));
          return Err((arg1, arg2));
        }
        // safety: We checked that the lengths were compatible, so
        // `Tensor` won't panic. Further, at least one of `arg1` or
        // `arg2` is a vector (since the above cases would have
        // handled any situations where both are scalar), so the
        // division operator here merely simplifies the expression and
        // does NOT invoke Number::div or ComplexNumber::div, so
        // division by zero will NOT panic here.
        Ok(Expr::from(arg1 / arg2))
      })
    )
    .add_case(
      // Interval division (currently a trap case)
      //
      // TODO: Implement this once we have infinities (Issue #4)
      builder::arity_two().both_of_type(ExprToIntervalLike).and_then(|arg1, arg2, ctx| {
        ctx.errors.push(SimplifierError::custom_error("/", "Interval division is not currently supported"));
        Err((arg1, arg2))
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
    .build()
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
          context.errors.push(SimplifierError::division_by_zero("^"));
          return Err((arg1, arg2));
        }
        let power = pow_real(arg1, arg2);
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
    .add_case(
      // Vector to scalar power function
      //
      // (We just distribute scalar exponents over the elements of the
      // vector)
      builder::arity_two().of_types(ExprToVector, ExprToComplex).and_then(|arg1, arg2, _| {
        let result = arg1.map(|elem| Expr::call("^", vec![elem, Expr::from(arg2.clone())]));
        Ok(result.into_expr())
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
      // Trap case: Complex numbers
      builder::arity_two().both_of_type(ExprToComplex).and_then(|arg1, arg2, context| {
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
      // Negation of a vector
      builder::arity_one().of_type(ExprToVector).and_then(|arg, _| {
        let result = arg.map(|e| Expr::call("negate", vec![e]));
        Ok(result.into_expr())
      })
    )
    .add_case(
      // Negation of an interval
      builder::arity_one().of_type(expr_to_interval()).and_then(|arg, _| {
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
        Ok(Expr::from(arg.abs()))
      })
    )
    .add_case(
      // Norm of a vector
      builder::arity_one().of_type(ExprToVector).and_then(|arg, _| {
        Ok(vector_norm(arg))
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
      // Real number signum
      builder::arity_one().of_type(expr_to_number()).and_then(|arg, _| {
        Ok(Expr::from(arg.signum()))
      })
    )
    .add_case(
      // Complex number signum
      builder::arity_one().of_type(ExprToComplex).and_then(|arg, _| {
        let arg = ComplexNumber::from(arg);
        if arg.is_zero() {
          // Corner case
          Ok(Expr::from(arg))
        } else {
          let abs = ComplexNumber::from_real(Number::from(arg.abs()));
          Ok(Expr::from(arg / abs))
        }
      })
    )
    .add_case(
      // Normalized vector
      builder::arity_one().of_type(ExprToVector).and_then(|arg, _| {
        let norm = vector_norm(arg.clone());
        Ok(Expr::call("/", vec![
          arg.into(),
          Expr::call("||", vec![norm, Expr::from(Number::from(1))]), // Corner case: If vector is zero, return zero
        ]))
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
