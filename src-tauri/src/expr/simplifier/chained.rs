
use super::base::{Simplifier, SimplifierContext};
use super::identity::IdentitySimplifier;
use crate::expr::Expr;

pub struct ChainedSimplifier<'a, 'b> {
  left: Box<dyn Simplifier + 'a>,
  right: Box<dyn Simplifier + 'b>,
}

impl<'a, 'b> ChainedSimplifier<'a, 'b> {
  pub fn new(left: Box<dyn Simplifier + 'a>, right: Box<dyn Simplifier + 'b>) -> Self {
    ChainedSimplifier {
      left,
      right,
    }
  }
}

impl ChainedSimplifier<'static, 'static> {
  pub fn several<'c>(args: impl IntoIterator<Item = Box<dyn Simplifier + 'c>>) -> Box<dyn Simplifier + 'c> {
    args.into_iter()
      .reduce(|a, b| Box::new(ChainedSimplifier::new(a, b)))
      .unwrap_or_else(|| Box::new(IdentitySimplifier))
  }
}

impl<'a, 'b> Simplifier for ChainedSimplifier<'a, 'b> {
  fn simplify_expr(&self, expr: Expr, ctx: &mut SimplifierContext) -> Expr {
    let expr = self.left.simplify_expr(expr, ctx);
    self.right.simplify_expr(expr, ctx)
  }

  fn simplify_expr_part(&self, expr: Expr, ctx: &mut SimplifierContext) -> Expr {
    let expr = self.left.simplify_expr_part(expr, ctx);
    self.right.simplify_expr_part(expr, ctx)
  }
}
