
use crate::expr::Expr;
use crate::expr::walker::postorder_walk_ok;
use crate::errorlist::ErrorList;
use super::error::SimplifierError;

/// A simplifier provides a way to simplify mathematical expressions
/// according to some rules. A simplifier is required to supply the
/// `simplify_expr_part` method, which is called by `simplify_expr` on
/// each expression in a tree, in post-order. That is, each leaf of a
/// node in the expression tree will be simplified before the node
/// itself.
pub trait Simplifier {
  fn simplify_expr_part(&self, expr: Expr, errors: &mut ErrorList<SimplifierError>) -> Expr;

  fn simplify_expr(&self, expr: Expr, errors: &mut ErrorList<SimplifierError>) -> Expr {
    postorder_walk_ok(expr, |e| self.simplify_expr_part(e, errors))
  }
}
