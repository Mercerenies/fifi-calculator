
use super::base::{Simplifier, SimplifierContext};
use crate::expr::Expr;
use crate::expr::walker::postorder_walk_ok;

#[derive(Debug)]
pub struct RepeatedSimplifier<S> {
  inner: S,
  times: usize,
}

impl<S> RepeatedSimplifier<S> {
  pub fn new(inner: S, times: usize) -> RepeatedSimplifier<S> {
    RepeatedSimplifier { inner, times }
  }
}

impl<S: Simplifier> Simplifier for RepeatedSimplifier<S> {
  fn simplify_expr(&self, mut expr: Expr, ctx: &mut SimplifierContext) -> Expr {
    for _ in 0..self.times {
      expr = postorder_walk_ok(expr, |e| self.simplify_expr_part(e, ctx));
    }
    expr
  }

  fn simplify_expr_part(&self, expr: Expr, ctx: &mut SimplifierContext) -> Expr {
    self.inner.simplify_expr_part(expr, ctx)
  }
}
