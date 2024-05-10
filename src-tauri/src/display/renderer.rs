
use crate::expr::Expr;
use crate::expr::atom::Atom;

/// A display renderer provides a mechanism to convert Exprs into HTML
/// code for display within the frontend.
pub trait Renderer {
  fn to_html(&self, out: &mut String, expr: &Expr);
}

#[derive(Clone, Debug)]
pub struct DefaultRenderer;

impl Renderer for DefaultRenderer {
  fn to_html(&self, out: &mut String, expr: &Expr) {
    match expr {
      Expr::Atom(Atom::Number(n)) => {
        out.push_str(&n.to_string());
      }
    }
  }
}
