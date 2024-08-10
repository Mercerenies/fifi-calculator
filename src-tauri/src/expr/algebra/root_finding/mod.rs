
pub mod bisection;
pub mod newton;
pub mod secant;

use crate::expr::Expr;
use crate::expr::vector::Vector;
use crate::expr::number::{Number, ComplexNumber, ComplexLike};
use crate::expr::prisms::{ExprToComplex, expr_to_typed_vector, expr_to_number};
use crate::util::prism::{Prism, PrismExt};

use either::Either;

/// Valid input types to our various root-finding algorithms.
#[derive(Debug, Clone)]
pub enum RootFindingInput {
  Real(Number),
  Complex(ComplexNumber),
  PairOfReals(PairOfReals),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PairOfReals(pub Number, pub Number);

/// Prism which accepts vectors of length two, where each component is
/// a real number.
#[derive(Debug, Clone)]
pub struct VecToPairOfReals;

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
  use Either::{Left, Right};
  ExprToComplex.or(expr_to_pair_of_reals())
    .rmap(|complex_like| match complex_like {
      Left(ComplexLike::Real(r)) => RootFindingInput::Real(r),
      Left(ComplexLike::Complex(c)) => RootFindingInput::Complex(c),
      Right(pair) => RootFindingInput::PairOfReals(pair),
    }, |input| match input {
      RootFindingInput::Real(r) => Left(ComplexLike::Real(r)),
      RootFindingInput::Complex(c) => Left(ComplexLike::Complex(c)),
      RootFindingInput::PairOfReals(pair) => Right(pair),
    })
}

/// Prism which accepts vectors of length two, where each component
/// is a real number.
pub fn expr_to_pair_of_reals() -> impl Prism<Expr, PairOfReals> + Clone {
  expr_to_typed_vector(expr_to_number()).composed(VecToPairOfReals)
}

impl<T: Into<Expr>> FoundRoot<T> {
  pub fn into_vec(self) -> Vector {
    Vector::from(vec![self.value.into(), self.final_epsilon.into()])
  }

  pub fn into_expr(self) -> Expr {
    self.into_vec().into()
  }
}

impl Prism<Vec<Number>, PairOfReals> for VecToPairOfReals {
  fn narrow_type(&self, vec: Vec<Number>) -> Result<PairOfReals, Vec<Number>> {
    if vec.len() == 2 {
      let [x, y] = vec.try_into().unwrap();
      Ok(PairOfReals(x, y))
    } else {
      Err(vec)
    }
  }

  fn widen_type(&self, pair: PairOfReals) -> Vec<Number> {
    vec![pair.0, pair.1]
  }
}
