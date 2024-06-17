
use super::base::{Simplifier, SimplifierContext};
use crate::expr::Expr;

#[derive(Debug, Clone, Copy)]
pub struct IdentitySimplifier;

impl Simplifier for IdentitySimplifier {
  fn simplify_expr_part(&self, expr: Expr, _: &mut SimplifierContext) -> Expr {
    expr
  }
}
