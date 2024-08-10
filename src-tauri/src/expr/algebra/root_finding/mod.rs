
pub mod newton;
pub mod secant;

use crate::expr::Expr;
use crate::expr::vector::Vector;
use crate::expr::number::{Number, ComplexNumber, ComplexLike};
use crate::expr::prisms::ExprToComplex;
use crate::util::prism::{Prism, PrismExt};

/// Valid input types to our various root-finding algorithms.
#[derive(Debug, Clone)]
pub enum RootFindingInput {
  Real(Number),
  Complex(ComplexNumber),
}

/// A root found using one of our supported root finding algorithms.
#[derive(Debug, Clone)]
pub struct FoundRoot<T> {
  pub value: T,
  pub final_epsilon: f64,
}

/// Prism which parses an expression as [`RootFindingInput`].
/// This prism accepts only real or complex number
/// literals.
pub fn expr_to_root_finding_input() -> impl Prism<Expr, RootFindingInput> + Clone {
  ExprToComplex
    .rmap(|complex_like| match complex_like {
      ComplexLike::Real(r) => RootFindingInput::Real(r),
      ComplexLike::Complex(c) => RootFindingInput::Complex(c),
    }, |input| match input {
      RootFindingInput::Real(r) => ComplexLike::Real(r),
      RootFindingInput::Complex(c) => ComplexLike::Complex(c),
    })
}

impl<T: Into<Expr>> FoundRoot<T> {
  pub fn into_vec(self) -> Vector {
    Vector::from(vec![self.value.into(), self.final_epsilon.into()])
  }

  pub fn into_expr(self) -> Expr {
    self.into_vec().into()
  }
}
