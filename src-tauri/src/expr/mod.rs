
pub mod atom;
pub mod number;
pub mod walker;

#[derive(Debug, Clone)]
pub enum Expr {
  Atom(atom::Atom),
  Call(String, Vec<Expr>),
}

impl Expr {

  /// Convenience constructor for [Expr::Call].
  pub fn call(name: &str, args: Vec<Expr>) -> Expr {
    Expr::Call(name.to_string(), args)
  }

}

impl From<atom::Atom> for Expr {
  fn from(a: atom::Atom) -> Expr {
    Expr::Atom(a)
  }
}
