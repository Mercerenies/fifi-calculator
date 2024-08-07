
pub mod algebra;
pub mod atom;
pub mod basic_parser;
pub mod calculus;
pub mod function;
pub mod incomplete;
pub mod interval;
pub mod literal;
pub mod number;
pub mod ordering;
pub mod prisms;
pub mod simplifier;
pub mod tokenizer;
pub mod units;
pub mod var;
pub mod vector;
pub mod walker;

use atom::Atom;
use var::Var;
use var::table::VarTable;
use number::{Number, ComplexNumber, Quaternion};
use crate::util::prism::ErrorWithPayload;

use thiserror::Error;
use num::{Zero, One, BigInt};
use serde::{Serialize, Deserialize};

use std::mem;
use std::fmt::{self, Display, Formatter};
use std::collections::HashSet;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum Expr {
  Atom(Atom),
  Call(String, Vec<Expr>),
}

#[derive(Debug, Clone, PartialEq, Eq, Error)]
#[error("Failed to parse expression '{original_expr}' as {target_type}")]
pub struct TryFromExprError {
  pub target_type: String,
  pub original_expr: Expr,
}

impl Expr {
  pub fn zero() -> Expr {
    Expr::Atom(Number::zero().into())
  }

  pub fn one() -> Expr {
    Expr::Atom(Number::one().into())
  }

  /// Returns true if this expression is literally equal to zero. This
  /// returns true for all representations of zero, including integer,
  /// floating, and complex representations.
  ///
  /// This method never attempts any simplifications, so it returns
  /// false for expressions which are clearly _mathematically_ zero
  /// but are not literally zero, such as `0 * x`.
  pub fn is_zero(&self) -> bool {
    match self {
      Expr::Atom(Atom::Number(n)) => n.is_zero(),
      Expr::Call(f, args) => {
        if f == ComplexNumber::FUNCTION_NAME && args.len() == 2 {
          args[0].is_zero() && args[1].is_zero()
        } else if f == Quaternion::FUNCTION_NAME && args.len() == 4 {
          args[0].is_zero() && args[1].is_zero() && args[2].is_zero() && args[3].is_zero()
        } else {
          false
        }
      }
      _ => false,
    }
  }

  /// Returns true if this expression is literally equal to one. This
  /// returns true for all representations of one, including integer,
  /// floating, and complex representations.
  pub fn is_one(&self) -> bool {
    match self {
      Expr::Atom(Atom::Number(n)) => n.is_one(),
      Expr::Call(f, args) => {
        if f == ComplexNumber::FUNCTION_NAME && args.len() == 2 {
          args[0].is_one() && args[1].is_zero()
        } else if f == Quaternion::FUNCTION_NAME && args.len() == 4 {
          args[0].is_one() && args[1].is_zero() && args[2].is_zero() && args[3].is_zero()
        } else {
          false
        }
      }
      _ => false,
    }
  }

  /// Equivalent to [`Expr::from`] but can be used to make the
  /// intention clearer.
  pub fn string(s: impl Into<String>) -> Expr {
    Expr::from(s.into())
  }

  pub fn var(name: &str) -> Option<Expr> {
    Var::new(name).map(|v| Expr::Atom(v.into()))
  }

  /// Convenience constructor for [Expr::Call].
  pub fn call(name: impl Into<String>, args: Vec<Expr>) -> Expr {
    Expr::Call(name.into(), args)
  }

  pub fn mutate<F>(&mut self, f: F)
  where F: FnOnce(Expr) -> Expr {
    // Temporarily replace with a meaningless placeholder value.
    let placeholder = Expr::Atom(Atom::Number(0f64.into())); // Ugh... simplest non-allocating value I have
    let original_value = mem::replace(self, placeholder);
    *self = f(original_value);
  }

  pub fn mutate_failable<F, E>(&mut self, f: F) -> Result<(), E>
  where F: FnOnce(Expr) -> Result<Expr, E> {
    // Temporarily replace with a meaningless placeholder value.
    let placeholder = Expr::Atom(Atom::Number(0f64.into())); // Ugh... simplest non-allocating value I have
    let original_value = mem::replace(self, placeholder);
    *self = f(original_value)?;
    Ok(())
  }

  pub fn substitute_var(self, var: Var, value: Expr) -> Self {
    walker::postorder_walk_ok(self, |expr| {
      if let Expr::Atom(Atom::Var(v)) = expr {
        if v == var {
          value.clone()
        } else {
          Expr::Atom(Atom::Var(v))
        }
      } else {
        expr
      }
    })
  }

  pub fn substitute_vars(self, vars: &VarTable<Expr>) -> Self {
    walker::postorder_walk_ok(self, |expr| {
      if let Expr::Atom(Atom::Var(v)) = expr {
        match vars.get(&v) {
          None => Expr::Atom(Atom::Var(v)),
          Some(value) => value.clone(),
        }
      } else {
        expr
      }
    })
  }

  pub fn free_vars(self) -> HashSet<Var> {
    let mut result = HashSet::new();
    walker::postorder_walk_ok(self, |expr| {
      if let Expr::Atom(Atom::Var(v)) = expr {
        result.insert(v.clone());
        Expr::Atom(Atom::Var(v))
      } else {
        expr
      }
    });
    result
  }
}

impl TryFromExprError {
  pub fn new(target_type: impl Into<String>, original_expr: Expr) -> Self {
    Self {
      target_type: target_type.into(),
      original_expr,
    }
  }

  pub fn with_type_name(mut self, new_target_type: impl Into<String>) -> Self {
    self.target_type = new_target_type.into();
    self
  }
}

impl ErrorWithPayload<Expr> for TryFromExprError {
  fn recover_payload(self) -> Expr {
    self.original_expr
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
            write!(f, ", ")?;
          }
          first = false;
          write!(f, "{arg}")?;
        }
        write!(f, ")")
      }
    }
  }
}

impl From<Atom> for Expr {
  fn from(a: Atom) -> Expr {
    Expr::Atom(a)
  }
}

impl From<String> for Expr {
  fn from(s: String) -> Expr {
    Expr::Atom(s.into())
  }
}

impl From<&str> for Expr {
  fn from(s: &str) -> Expr {
    String::from(s).into()
  }
}

impl From<Var> for Expr {
  fn from(v: Var) -> Expr {
    Expr::Atom(v.into())
  }
}

impl From<Number> for Expr {
  fn from(n: Number) -> Expr {
    Expr::Atom(n.into())
  }
}

impl From<ComplexNumber> for Expr {
  fn from(z: ComplexNumber) -> Expr {
    let (real, imag) = z.into_parts();
    Expr::call(ComplexNumber::FUNCTION_NAME, vec![Expr::from(real), Expr::from(imag)])
  }
}

impl From<Quaternion> for Expr {
  fn from(q: Quaternion) -> Expr {
    let (r, i, j, k) = q.into_parts();
    Expr::call(Quaternion::FUNCTION_NAME, vec![
      Expr::from(r),
      Expr::from(i),
      Expr::from(j),
      Expr::from(k),
    ])
  }
}

impl From<BigInt> for Expr {
  fn from(b: BigInt) -> Expr {
    Expr::Atom(Atom::Number(b.into()))
  }
}

/// Booleans are represented in the expression language as the literal
/// integers zero and one.
impl From<bool> for Expr {
  fn from(b: bool) -> Expr {
    Expr::from(Number::from(b))
  }
}

impl From<i64> for Expr {
  fn from(i: i64) -> Expr {
    Expr::Atom(i.into())
  }
}

impl From<f64> for Expr {
  fn from(f: f64) -> Expr {
    Expr::Atom(f.into())
  }
}

impl TryFrom<Expr> for Number {
  type Error = TryFromExprError;

  fn try_from(e: Expr) -> Result<Self, Self::Error> {
    match e {
      Expr::Atom(Atom::Number(n)) => Ok(n),
      e => Err(TryFromExprError::new("Number", e)),
    }
  }
}

impl TryFrom<Expr> for String {
  type Error = TryFromExprError;

  fn try_from(e: Expr) -> Result<Self, Self::Error> {
    match e {
      Expr::Atom(Atom::String(s)) => Ok(s),
      e => Err(TryFromExprError::new("String", e)),
    }
  }
}

impl TryFrom<Expr> for Var {
  type Error = TryFromExprError;

  fn try_from(e: Expr) -> Result<Self, Self::Error> {
    match e {
      Expr::Atom(Atom::Var(v)) => Ok(v),
      e => Err(TryFromExprError::new("Var", e)),
    }
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  fn var(name: &str) -> Expr {
    Expr::var(name).unwrap()
  }

  #[test]
  fn test_var_substitute_with_no_variables() {
    let expr = Expr::call("+", vec![Expr::from(1), Expr::from(2)]);
    let new_expr = expr.clone().substitute_var(Var::new("x").unwrap(), Expr::from(999));
    assert_eq!(new_expr, expr);
  }

  #[test]
  fn test_var_substitute_with_non_matching_var() {
    let expr = Expr::call("+", vec![var("y"), Expr::from(2)]);
    let new_expr = expr.clone().substitute_var(Var::new("x").unwrap(), Expr::from(999));
    assert_eq!(new_expr, expr);
  }

  #[test]
  fn test_var_substitute_with_non_matching_vars() {
    let expr = Expr::call(
      "+",
      vec![
        var("y"),
        Expr::call("*", vec![var("z"), var("x1")]),
      ],
    );
    let new_expr = expr.clone().substitute_var(Var::new("x").unwrap(), Expr::from(999));
    assert_eq!(new_expr, expr);
  }

  #[test]
  fn test_var_substitute_with_vars() {
    let expr = Expr::call("+", vec![var("y"), var("x")]);
    let new_expr = expr.substitute_var(Var::new("x").unwrap(), Expr::from(999));
    assert_eq!(
      new_expr,
      Expr::call("+", vec![var("y"), Expr::from(999)]),
    );
  }

  #[test]
  fn test_var_substitute_with_var_containing_itself() {
    let expr = Expr::call("+", vec![var("x"), Expr::from(1)]);
    let new_expr = expr.substitute_var(
      Var::new("x").unwrap(),
      Expr::call("+", vec![var("x"), Expr::from(2)]),
    );
    assert_eq!(
      new_expr,
      Expr::call("+", vec![Expr::call("+", vec![var("x"), Expr::from(2)]), Expr::from(1)]),
    );
  }

  #[test]
  fn test_var_substitute_with_same_var_twice() {
    let expr = Expr::call("+", vec![var("x"), var("x")]);
    let new_expr = expr.substitute_var(Var::new("x").unwrap(), Expr::from(999));
    assert_eq!(
      new_expr,
      Expr::call("+", vec![Expr::from(999), Expr::from(999)]),
    );
  }

  #[test]
  fn test_multi_var_substitute() {
    let mut vars = VarTable::new();
    vars.insert(Var::new("x").unwrap(), Expr::from(1));
    vars.insert(Var::new("y").unwrap(), Expr::from(2));

    let expr = Expr::call("+", vec![var("y"), var("x")]);
    let new_expr = expr.substitute_vars(&vars);
    assert_eq!(
      new_expr,
      Expr::call("+", vec![Expr::from(2), Expr::from(1)]),
    );
  }

  #[test]
  fn test_multi_var_substitute_as_each_other() {
    let mut vars = VarTable::new();
    vars.insert(Var::new("x").unwrap(), Expr::call("+", vec![var("y"), Expr::from(1)]));
    vars.insert(Var::new("y").unwrap(), Expr::call("+", vec![var("x"), Expr::from(2)]));

    let expr = Expr::call("+", vec![var("y"), var("x")]);
    let new_expr = expr.substitute_vars(&vars);
    assert_eq!(
      new_expr,
      Expr::call("+", vec![
        Expr::call("+", vec![var("x"), Expr::from(2)]),
        Expr::call("+", vec![var("y"), Expr::from(1)]),
      ]),
    );
  }
}
