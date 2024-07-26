
use crate::expr::Expr;
use crate::expr::walker::postorder_walk_ok;
use crate::errorlist::ErrorList;
use crate::mode::calculation::CalculationMode;
use super::error::SimplifierError;

/// A simplifier provides a way to simplify mathematical expressions
/// according to some rules. A simplifier is required to supply the
/// `simplify_expr_part` method, which is called by `simplify_expr` on
/// each expression in a tree, in post-order. That is, each leaf of a
/// node in the expression tree will be simplified before the node
/// itself.
pub trait Simplifier {
  /// Function to simplify a portion of an expression. Callers should
  /// generally invoke [`Simplifier::simplify_expr`] instead of
  /// calling this function directly. The former will invoke the
  /// latter recursively.
  fn simplify_expr_part(&self, expr: Expr, ctx: &mut SimplifierContext) -> Expr;

  /// Calls [`Simplifier::simplify_expr_part`] in a post-order
  /// traversal for each node in the expression tree.
  ///
  /// The default implementation runs only one time (in post-order) on
  /// the whole tree, but other simplifiers may choose to run multiple
  /// times.
  fn simplify_expr(&self, expr: Expr, ctx: &mut SimplifierContext) -> Expr {
    postorder_walk_ok(expr, |e| self.simplify_expr_part(e, ctx))
  }
}

pub struct SimplifierContext<'a, 'b> {
  pub base_simplifier: &'a dyn Simplifier,
  pub calculation_mode: CalculationMode,
  pub errors: &'b mut ErrorList<SimplifierError>,
}

impl<'a, S> Simplifier for &'a S
where S: Simplifier + ?Sized {
  fn simplify_expr_part(&self, expr: Expr, ctx: &mut SimplifierContext) -> Expr {
    (**self).simplify_expr_part(expr, ctx)
  }
}
