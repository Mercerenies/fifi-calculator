
use super::InfiniteConstant;
use crate::expr::Expr;
use crate::util::prism::Prism;

#[derive(Debug, Clone)]
pub struct ExprToInfinity;

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
