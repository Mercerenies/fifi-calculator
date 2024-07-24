
use super::{InfiniteConstant, SignedInfinity, UnboundedNumber};
use crate::expr::Expr;
use crate::expr::prisms::expr_to_number;
use crate::util::prism::{Prism, PrismExt, Conversion};

use either::Either;

#[derive(Debug, Clone)]
pub struct ExprToInfinity;

pub fn infinity_to_signed_infinity() -> Conversion<InfiniteConstant, SignedInfinity> {
  Conversion::new()
}

pub fn expr_to_signed_infinity() -> impl Prism<Expr, SignedInfinity> + Clone {
  ExprToInfinity.composed(infinity_to_signed_infinity())
}

pub fn expr_to_unbounded_number() -> impl Prism<Expr, UnboundedNumber> + Clone {
  expr_to_signed_infinity().or(expr_to_number()).rmap(|either| {
    match either {
      Either::Left(inf) => UnboundedNumber::Infinite(inf),
      Either::Right(n) => UnboundedNumber::Finite(n),
    }
  }, |unbounded| {
    match unbounded {
      UnboundedNumber::Infinite(inf) => Either::Left(inf),
      UnboundedNumber::Finite(n) => Either::Right(n),
    }
  })
}

impl Prism<Expr, InfiniteConstant> for ExprToInfinity {
  fn narrow_type(&self, input: Expr) -> Result<InfiniteConstant, Expr> {
    InfiniteConstant::ALL.into_iter().find(|c| input == Expr::from(c)).ok_or(input)
  }

  fn widen_type(&self, input: InfiniteConstant) -> Expr {
    input.into()
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn test_roundtrip_all_infinity_constants() {
    let prism = ExprToInfinity;
    for constant in InfiniteConstant::ALL {
      let expr = prism.widen_type(constant);
      assert_eq!(prism.narrow_type(expr).unwrap(), constant);
    }
  }
}
