
//! Simple functions that apply to, or extract parts of, complex
//! numbers.


//! Evaluation rules for transcendental and trigonometric functions.

use crate::expr::Expr;
use crate::expr::function::Function;
use crate::expr::function::table::FunctionTable;
use crate::expr::function::builder::{self, FunctionBuilder};
use crate::expr::prisms::{self, ExprToComplex};
use crate::expr::number::ComplexNumber;
use crate::expr::algebra::infinity::InfiniteConstant;

use std::f64::consts::PI;

pub fn append_complex_functions(table: &mut FunctionTable) {
  table.insert(conjugate());
  table.insert(arg());
  table.insert(re());
  table.insert(im());
}

pub fn conjugate() -> Function {
  FunctionBuilder::new("conj")
    .add_case(
      // Conjugate of a complex number
      builder::arity_one().of_type(ExprToComplex).and_then(|arg, _| {
        Ok(Expr::from(ComplexNumber::from(arg).conj()))
      })
    )
    .add_case(
      // Conjugate of infinity constant
      builder::arity_one().of_type(prisms::ExprToInfinity).and_then(|arg, _| {
        Ok(Expr::from(arg))
      })
    )
    .build()
}

pub fn arg() -> Function {
  FunctionBuilder::new("arg")
    .add_case(
      // Argument (phase) of a complex number
      builder::arity_one().of_type(ExprToComplex).and_then(|arg, _| {
        let angle = ComplexNumber::from(arg).angle();
        Ok(Expr::from(angle.0))
      })
    )
    .add_case(
      // Argument (phase) of infinity
      builder::arity_one().of_type(prisms::ExprToInfinity).and_then(|arg, _| {
        let phase = match arg {
          InfiniteConstant::PosInfinity => Expr::zero(),
          InfiniteConstant::NegInfinity => Expr::from(PI),
          InfiniteConstant::NotANumber | InfiniteConstant::UndirInfinity => Expr::from(InfiniteConstant::NotANumber),
        };
        Ok(phase)
      })
    )
    .build()
}

pub fn re() -> Function {
  FunctionBuilder::new("re")
    .add_case(
      // Real part of a complex number
      builder::arity_one().of_type(ExprToComplex).and_then(|arg, _| {
        let arg = ComplexNumber::from(arg);
        Ok(arg.into_parts().0.into())
      })
    )
    .add_case(
      // Real part of infinity constant
      builder::arity_one().of_type(prisms::ExprToInfinity).and_then(|arg, _| {
        if arg == InfiniteConstant::NotANumber || arg == InfiniteConstant::UndirInfinity {
          Ok(Expr::from(InfiniteConstant::NotANumber))
        } else {
          Ok(Expr::from(arg))
        }
      })
    )
    .build()
}

pub fn im() -> Function {
  FunctionBuilder::new("im")
    .add_case(
      // Imaginary part of a complex number
      builder::arity_one().of_type(ExprToComplex).and_then(|arg, _| {
        let arg = ComplexNumber::from(arg);
        Ok(arg.into_parts().1.into())
      })
    )
    .add_case(
      // Imaginary part of infinity constant
      builder::arity_one().of_type(prisms::ExprToInfinity).and_then(|arg, _| {
        if arg == InfiniteConstant::NotANumber || arg == InfiniteConstant::UndirInfinity {
          Ok(Expr::from(InfiniteConstant::NotANumber))
        } else {
          Ok(Expr::zero())
        }
      })
    )
    .build()
}
