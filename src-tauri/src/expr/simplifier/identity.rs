
use super::base::Simplifier;
use super::error::SimplifierError;
use crate::errorlist::ErrorList;
use crate::expr::Expr;

#[derive(Debug, Clone, Copy)]
pub struct IdentitySimplifier;

impl Simplifier for IdentitySimplifier {
  fn simplify_expr_part(&self, expr: Expr, _: &mut ErrorList<SimplifierError>) -> Expr {
    expr
  }
}
