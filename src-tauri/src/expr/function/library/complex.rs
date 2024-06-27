
//! Simple functions that apply to, or extract parts of, complex
//! numbers.


//! Evaluation rules for transcendental and trigonometric functions.

use crate::expr::Expr;
use crate::expr::simplifier::error::SimplifierError;
use crate::expr::function::Function;
use crate::expr::function::table::FunctionTable;
use crate::expr::function::builder::{self, FunctionBuilder};
use crate::expr::prisms::{self, ExprToNumber, ExprToComplex};
use crate::expr::number::{Number, ComplexNumber, pow_real, pow_complex};

use num::Zero;

use std::f64::consts;

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
    .build()
}
