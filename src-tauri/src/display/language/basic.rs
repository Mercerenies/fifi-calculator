
use super::LanguageMode;
use crate::expr::Expr;
use crate::expr::atom::Atom;

use std::marker::PhantomData;

#[derive(Clone, Debug, Default)]
pub struct BasicLanguageMode {
  _private: PhantomData<()>,
}

impl BasicLanguageMode {
  pub fn new() -> Self {
    Self::default()
  }

  pub fn with_default_operators() -> Self {
    // TODO Add operators
    Self::default()
  }

  fn function_call_to_html(&self, out: &mut String, f: &str, args: &[Expr]) {
    let mut first = true;
    out.push_str(f);
    out.push('(');
    args.iter().for_each(|e| {
      if !first {
        out.push_str(", ");
      }
      first = false;
      self.to_html(out, e);
    });
    out.push(')');
  }
}

impl LanguageMode for BasicLanguageMode {
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
        self.function_call_to_html(out, f, args);
      }
    }
  }
}
