
//! This module defines a total ordering on the [`Expr`] type. This
//! ordering has the following properties.
//!
//! * It is consistent with the `Eq` instance on `Expr`.
//!
//! * Real numbers, as well as positive and negative infinity, compare
//! using the typical ordering. Non-signed infinities are sorted
//! arbitrarily.
//!
//! * Strings compare lexicographically.
//!
//! * Vectors compare lexicographically, using this same ordering on
//! the elements.
//!
//! * Like-kinded intervals of unbounded numbers compare by their
//! lower bound first, then their upper bound.
//!
//! * Variables which do NOT represent infinity constants are sorted
//! alphabetically.
//!
//! * Variables which do NOT represent infinity constants are greater
//! than any real number or infinity constant.

use super::Expr;
use super::var::Var;
use super::atom::Atom;
use super::number::Number;
use super::algebra::infinity::InfiniteConstant;
use crate::util::cmp_iter_by;

use std::str::FromStr;
use std::cmp::Ordering;

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct OrderedExpr<'a> {
  data: OrderedExprImpl<'a>,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
enum OrderedExprImpl<'a> {
  NegInfinity,
  Number(&'a Number),
  PosInfinity,
  UndirInfinity,
  NotANumber,
  String(&'a str),
  Var(&'a Var),
  Call(&'a str, OrderedExprSlice<'a>),
}

/// Type which implements ordering for a slice of `Expr` via this
/// module, without having to clone any of the inner expressions or
/// the vector structure.
#[derive(Debug, Clone, PartialEq, Eq)]
struct OrderedExprSlice<'a> {
  elems: &'a [Expr],
}

impl<'a> OrderedExpr<'a> {
  pub fn new(e: &'a Expr) -> Self {
    let data = match e {
      Expr::Atom(Atom::Number(n)) => OrderedExprImpl::Number(n),
      Expr::Atom(Atom::String(s)) => OrderedExprImpl::String(s),
      Expr::Atom(Atom::Var(v)) => {
        if let Ok(inf) = InfiniteConstant::from_str(v.as_str()) {
          OrderedExpr::from(inf).data
        } else {
          OrderedExprImpl::Var(v)
        }
      }
      Expr::Call(function_name, args) => {
        // Detect negative infinity (TODO (HACK): Do this using
        // crate::expr::algebra::infinity or somewhere sensible)
        if function_name == "negate" && args.len() == 1 && args[0] == Expr::var("inf").unwrap() {
          OrderedExprImpl::NegInfinity
        } else {
          OrderedExprImpl::Call(function_name, OrderedExprSlice { elems: args })
        }
      }
    };
    OrderedExpr { data }
  }
}

impl<'a> PartialOrd for OrderedExprSlice<'a> {
  fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
    Some(self.cmp(other))
  }
}

impl<'a> Ord for OrderedExprSlice<'a> {
  fn cmp(&self, other: &Self) -> Ordering {
    cmp_iter_by(self.elems, other.elems, |a, b| cmp_expr(a, b))
  }
}

impl<'a> From<InfiniteConstant> for OrderedExpr<'a> {
  fn from(i: InfiniteConstant) -> Self {
    match i {
      InfiniteConstant::NegInfinity => OrderedExpr { data: OrderedExprImpl::NegInfinity },
      InfiniteConstant::PosInfinity => OrderedExpr { data: OrderedExprImpl::PosInfinity },
      InfiniteConstant::UndirInfinity => OrderedExpr { data: OrderedExprImpl::UndirInfinity },
      InfiniteConstant::NotANumber => OrderedExpr { data: OrderedExprImpl::NotANumber },
    }
  }
}

pub fn cmp_expr(a: &Expr, b: &Expr) -> Ordering {
  OrderedExpr::new(a).cmp(&OrderedExpr::new(b))
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn test_parse_neg_infinity() {
    let expr = Expr::call("negate", vec![Expr::var("inf").unwrap()]);
    assert_eq!(OrderedExpr::new(&expr), OrderedExpr { data: OrderedExprImpl::NegInfinity });
  }

  #[test]
  fn test_parse_number() {
    let expr = Expr::from(10);
    assert_eq!(OrderedExpr::new(&expr), OrderedExpr { data: OrderedExprImpl::Number(&Number::from(10)) });
  }

  #[test]
  fn test_parse_non_neg_infinities() {
    let expr = Expr::var("inf").unwrap();
    assert_eq!(OrderedExpr::new(&expr), OrderedExpr { data: OrderedExprImpl::PosInfinity });
    let expr = Expr::var("uinf").unwrap();
    assert_eq!(OrderedExpr::new(&expr), OrderedExpr { data: OrderedExprImpl::UndirInfinity });
    let expr = Expr::var("nan").unwrap();
    assert_eq!(OrderedExpr::new(&expr), OrderedExpr { data: OrderedExprImpl::NotANumber });
  }

  #[test]
  fn test_parse_string() {
    let expr = Expr::string("hello");
    assert_eq!(OrderedExpr::new(&expr), OrderedExpr { data: OrderedExprImpl::String("hello") });
  }

  #[test]
  fn test_parse_var() {
    let expr = Expr::var("x").unwrap();
    assert_eq!(OrderedExpr::new(&expr), OrderedExpr { data: OrderedExprImpl::Var(&Var::new("x").unwrap()) });
  }

  #[test]
  fn test_parse_call() {
    let expr = Expr::call("function_name", vec![Expr::from(10), Expr::from(20)]);
    assert_eq!(OrderedExpr::new(&expr), OrderedExpr {
      data: OrderedExprImpl::Call("function_name", OrderedExprSlice { elems: &[Expr::from(10), Expr::from(20)] }),
    });
  }
}
