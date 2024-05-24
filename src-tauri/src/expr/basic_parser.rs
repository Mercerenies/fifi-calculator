
use super::Expr;
use crate::parsing::shunting_yard::ShuntingYardDriver;
use crate::parsing::operator::Operator;

use std::convert::Infallible;

#[derive(Clone, Debug, Default)]
#[non_exhaustive]
pub struct ExprShuntingYardDriver {}

impl ExprShuntingYardDriver {
  pub fn new() -> Self {
    Self {}
  }
}

impl ShuntingYardDriver<Expr> for ExprShuntingYardDriver {
  type Output = Expr;
  type Error = Infallible;

  fn compile_scalar(&mut self, scalar: Expr) -> Result<Expr, Infallible> {
    Ok(scalar)
  }

  fn compile_bin_op(&mut self, left: Expr, oper: Operator, right: Expr) -> Result<Expr, Infallible> {
    Ok(Expr::call(oper.name(), vec![left, right]))
  }
}
