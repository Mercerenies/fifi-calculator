
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
      Expr::Call(f, args) => {
        // TODO Do smarter stuff for infix operators, etc.
        function_call_to_html(out, f, args);
      }
    }
  }
}

fn function_call_to_html(out: &mut String, f: &str, args: &[Expr]) {
  let mut first = true;
  out.push_str(f);
  out.push('(');
  args.iter().for_each(|e| {
    if !first {
      out.push_str(", ");
    }
    first = false;
    DefaultRenderer.to_html(out, e);
  });
  out.push(')');
}
