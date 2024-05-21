
use crate::expr::Expr;
use crate::expr::atom::Atom;

/// A language mode provides a mechanism to convert Exprs into HTML
/// code for display within the frontend.
pub trait LanguageMode {
  fn to_html(&self, out: &mut String, expr: &Expr);
}

#[derive(Clone, Debug)]
pub struct DefaultLanguageMode;

impl LanguageMode for DefaultLanguageMode {
  fn to_html(&self, out: &mut String, expr: &Expr) {
    match expr {
      Expr::Atom(Atom::Number(n)) => {
        out.push_str(&n.to_string());
      }
      Expr::Atom(Atom::Complex(z)) => {
        out.push_str(&z.to_string());
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
    DefaultLanguageMode.to_html(out, e);
  });
  out.push(')');
}
