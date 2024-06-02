
pub mod atom;
pub mod basic_parser;
pub mod number;
pub mod prisms;
pub mod simplifier;
pub mod tokenizer;
pub mod var;
pub mod walker;

use num::{Zero, One};

use std::mem;
use std::fmt::{self, Display, Formatter};

#[derive(Debug, Clone, PartialEq)]
pub enum Expr {
  Atom(atom::Atom),
  Call(String, Vec<Expr>),
}

#[derive(Debug, Clone)]
pub struct TryFromExprError {
  original_expr: Expr,
}

impl Expr {
  pub fn zero() -> Expr {
    Expr::Atom(number::Number::zero().into())
  }

  pub fn one() -> Expr {
    Expr::Atom(number::Number::one().into())
  }

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

/// This is a very simple display impl that doesn't handle situations
/// such as infix operators and is mainly used for getting reasonable
/// error output in case of a parse error. For regular program output,
/// consider using a [language
/// mode](crate::display::language::LanguageMode) instead.
impl Display for Expr {
  fn fmt(&self, f: &mut Formatter) -> fmt::Result {
    match self {
      Expr::Atom(a) => write!(f, "{a}"),
      Expr::Call(name, args) => {
        write!(f, "{name}(")?;
        let mut first = true;
        for arg in args {
          if !first {
            write!(f, ",")?;
          }
          first = false;
          write!(f, "{arg}")?;
        }
        write!(f, ")")
      }
    }
  }
}

impl From<atom::Atom> for Expr {
  fn from(a: atom::Atom) -> Expr {
    Expr::Atom(a)
  }
}

impl From<var::Var> for Expr {
  fn from(v: var::Var) -> Expr {
    Expr::Atom(v.into())
  }
}

impl From<number::Number> for Expr {
  fn from(n: number::Number) -> Expr {
    Expr::Atom(n.into())
  }
}

impl From<number::ComplexNumber> for Expr {
  fn from(z: number::ComplexNumber) -> Expr {
    Expr::Atom(z.into())
  }
}

impl From<i64> for Expr {
  fn from(i: i64) -> Expr {
    Expr::Atom(i.into())
  }
}

impl TryFrom<Expr> for number::Number {
  type Error = TryFromExprError;

  fn try_from(e: Expr) -> Result<Self, Self::Error> {
    match e {
      Expr::Atom(atom::Atom::Number(n)) => Ok(n),
      e => Err(TryFromExprError { original_expr: e }),
    }
  }
}
