
use super::Expr;
use super::number::Number;
use crate::util::prism::Prism;

/// Prism which downcasts an [`Expr`] to a contained [`Number`].
#[derive(Debug, Clone, Copy, Default)]
pub struct ExprToNumber;

impl ExprToNumber {
  pub fn new() -> Self {
    ExprToNumber
  }
}

impl Prism<Expr, Number> for ExprToNumber {
  fn narrow_type(&self, input: Expr) -> Result<Number, Expr> {
    Number::try_from(input).map_err(|err| err.original_expr)
  }

  fn widen_type(&self, input: Number) -> Expr {
    Expr::from(input)
  }
}
