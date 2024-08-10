
pub mod newton;

use crate::expr::Expr;
use crate::expr::vector::Vector;

#[derive(Debug, Clone)]
pub struct FoundRoot<T> {
  pub value: T,
  pub final_epsilon: f64,
}

impl<T: Into<Expr>> FoundRoot<T> {
  pub fn into_vec(self) -> Vector {
    Vector::from(vec![self.value.into(), self.final_epsilon.into()])
  }

  pub fn into_expr(self) -> Expr {
    self.into_vec().into()
  }
}
