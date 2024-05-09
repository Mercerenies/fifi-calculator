
pub mod number;
pub mod atom;

#[derive(Debug, Clone)]
pub enum Expr {
  Atom(atom::Atom),
}
