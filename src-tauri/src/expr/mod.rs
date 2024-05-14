
pub mod atom;
pub mod number;
pub mod simplifier;
pub mod walker;

use std::mem;

#[derive(Debug, Clone, PartialEq)]
pub enum Expr {
  Atom(atom::Atom),
  Call(String, Vec<Expr>),
}

#[derive(Debug, Clone)]
pub struct TryFromExprError;

impl Expr {
  /// Convenience constructor for [Expr::Call].
  pub fn call(name: &str, args: Vec<Expr>) -> Expr {
    Expr::Call(name.to_string(), args)
  }

  pub fn mutate<F>(&mut self, f: F)
  where F: FnOnce(Expr) -> Expr {
    // Temporarily replace with a meaningless placeholder value.
    let placeholder = Expr::Atom(atom::Atom::Number(0f64.into())); // Ugh... simplest non-allocating value I have
    let original_value = mem::replace(self, placeholder);
    *self = f(original_value);
  }
}

impl From<atom::Atom> for Expr {
  fn from(a: atom::Atom) -> Expr {
    Expr::Atom(a)
  }
}

impl From<number::Number> for Expr {
  fn from(n: number::Number) -> Expr {
    Expr::Atom(n.into())
  }
}

impl TryFrom<Expr> for number::Number {
  type Error = TryFromExprError;

  fn try_from(e: Expr) -> Result<Self, Self::Error> {
    match e {
      Expr::Atom(atom::Atom::Number(n)) => Ok(n),
      _ => Err(TryFromExprError),
    }
  }
}
