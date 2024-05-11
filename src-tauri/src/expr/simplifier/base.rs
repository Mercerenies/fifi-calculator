
use crate::expr::Expr;
use crate::expr::walker::postorder_walk_ok;
use crate::errorlist::ErrorList;
use super::error::SimplifierError;

pub trait Simplifier {
  fn simplify_expr_part(&self, expr: Expr, errors: &mut ErrorList<SimplifierError>) -> Expr;

  fn simplify_expr(&self, expr: Expr, errors: &mut ErrorList<SimplifierError>) -> Expr {
    postorder_walk_ok(expr, |e| self.simplify_expr_part(e, errors))
  }
}
