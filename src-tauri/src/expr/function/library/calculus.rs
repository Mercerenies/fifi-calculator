
//! Functions for doing basic calculus.

use crate::util::prism::Identity;
use crate::expr::Expr;
use crate::expr::function::Function;
use crate::expr::function::table::FunctionTable;
use crate::expr::function::builder::{self, FunctionBuilder};
use crate::expr::prisms;
use crate::expr::number::{Number, ComplexNumber, pow_real, pow_complex};
use crate::expr::calculus::differentiate;

use std::f64::consts;

pub fn append_calculus_functions(table: &mut FunctionTable) {
  table.insert(deriv());
}

pub fn deriv() -> Function {
  FunctionBuilder::new("deriv")
    .add_case(
      builder::arity_two().of_types(Identity::new(), prisms::ExprToVar).and_then(|expr, var, context| {
        todo!()
      })
    )
    .build()
}
