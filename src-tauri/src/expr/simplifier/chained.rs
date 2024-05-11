
use super::base::Simplifier;
use super::error::SimplifierError;
use super::identity::IdentitySimplifier;
use crate::errorlist::ErrorList;
use crate::expr::Expr;

pub struct ChainedSimplifier {
  left: Box<dyn Simplifier>,
  right: Box<dyn Simplifier>,
}

impl ChainedSimplifier {
  pub fn new(left: Box<dyn Simplifier>, right: Box<dyn Simplifier>) -> ChainedSimplifier {
    ChainedSimplifier {
      left,
      right,
    }
  }

  pub fn several(args: impl Iterator<Item = Box<dyn Simplifier>>) -> Box<dyn Simplifier> {
    args.reduce(|a, b| Box::new(ChainedSimplifier::new(a, b)))
      .unwrap_or_else(|| Box::new(IdentitySimplifier))
  }
}

impl Simplifier for ChainedSimplifier {
  fn simplify_expr_part(&self, expr: Expr, errors: &mut ErrorList<SimplifierError>) -> Expr {
    let expr = self.left.simplify_expr(expr, errors);
    self.right.simplify_expr(expr, errors)
  }
}
