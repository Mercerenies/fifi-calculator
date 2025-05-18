
//! This module defines a total ordering on the [`Expr`] type. This
//! ordering has the following properties.
//!
//! * It is consistent with the `Eq` instance on `Expr`.
//!
//! * Real numbers, as well as positive and negative infinity, compare
//!   using the typical ordering. Non-signed infinities are sorted
//!   arbitrarily.
//!
//! * Strings compare lexicographically.
//!
//! * Vectors compare lexicographically, using this same ordering on
//!   the elements.
//!
//! * Like-kinded intervals of unbounded numbers compare by their
//!   lower bound first, then their upper bound.
//!
//! * Variables which do NOT represent infinity constants are sorted
//!   alphabetically.
//!
//! * Variables which do NOT represent infinity constants are greater
//!   than any real number or infinity constant.

use super::Expr;
use super::var::Var;
use super::atom::Atom;
use super::number::Number;
use super::algebra::infinity::InfiniteConstant;
use super::algebra::factor::Factor;
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
///
/// An `OrderedExprSlice` can either borrow the slice of expressions
/// as a whole (`&[Expr]`) or can own a vector of borrowed expressions
/// (`Vec<&Expr>`). Both representations are treated equivalently by
/// the `Eq` and `Ord` instances.
#[derive(Debug, Clone)]
enum OrderedExprSlice<'a> {
  Borrowed { elems: &'a [Expr] },
  Owned { elems: Vec<&'a Expr> },
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
          OrderedExprImpl::Call(function_name, OrderedExprSlice::Borrowed { elems: args })
        }
      }
    };
    OrderedExpr { data }
  }

  /// Private helper to construct `OrderedExprImpl::Call` values.
  fn call(function_name: &'a str, args: Vec<&'a Expr>) -> Self {
    OrderedExpr {
      data: OrderedExprImpl::Call(function_name, OrderedExprSlice::Owned { elems: args }),
    }
  }
}

impl<'a> OrderedExprSlice<'a> {
  fn iter(&self) -> Box<dyn Iterator<Item = &'a Expr> + '_> {
    match self {
      OrderedExprSlice::Borrowed { elems } => Box::new(elems.iter()),
      OrderedExprSlice::Owned { elems } => Box::new(elems.iter().copied()),
    }
  }
}

impl PartialEq for OrderedExprSlice<'_> {
  fn eq(&self, other: &Self) -> bool {
    cmp_iter_by(self.iter(), other.iter(), |a, b| cmp_expr(a, b)) == Ordering::Equal
  }
}

impl Eq for OrderedExprSlice<'_> {}

impl PartialOrd for OrderedExprSlice<'_> {
  fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
    Some(self.cmp(other))
  }
}

impl Ord for OrderedExprSlice<'_> {
  fn cmp(&self, other: &Self) -> Ordering {
    cmp_iter_by(self.iter(), other.iter(), |a, b| cmp_expr(a, b))
  }
}

impl From<InfiniteConstant> for OrderedExpr<'_> {
  fn from(i: InfiniteConstant) -> Self {
    match i {
      InfiniteConstant::NegInfinity => OrderedExpr { data: OrderedExprImpl::NegInfinity },
      InfiniteConstant::PosInfinity => OrderedExpr { data: OrderedExprImpl::PosInfinity },
      InfiniteConstant::UndirInfinity => OrderedExpr { data: OrderedExprImpl::UndirInfinity },
      InfiniteConstant::NotANumber => OrderedExpr { data: OrderedExprImpl::NotANumber },
    }
  }
}

impl<'a> From<&'a Expr> for OrderedExpr<'a> {
  fn from(e: &'a Expr) -> Self {
    OrderedExpr::new(e)
  }
}

impl<'a> From<&'a Factor> for OrderedExpr<'a> {
  fn from(f: &'a Factor) -> Self {
    match f.exponent() {
      None => OrderedExpr::from(f.base()),
      Some(exp) => {
        OrderedExpr::call("^", vec![f.base(), exp])
      }
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
      data: OrderedExprImpl::Call(
        "function_name",
        OrderedExprSlice::Borrowed { elems: &[Expr::from(10), Expr::from(20)] },
      ),
    });
  }

  #[test]
  fn test_compare_slices_for_eq() {
    let two = Expr::from(2);
    let three = Expr::from(3);
    let slice1 = OrderedExprSlice::Borrowed { elems: &[Expr::from(2), Expr::from(3)] };
    let slice2 = OrderedExprSlice::Borrowed { elems: &[Expr::from(3), Expr::from(2)] };
    let slice3 = OrderedExprSlice::Owned { elems: vec![&two, &three] };
    let slice4 = OrderedExprSlice::Owned { elems: vec![&three, &two] };

    assert_eq!(slice1, slice1);
    assert_eq!(slice1, slice3);
    assert_eq!(slice3, slice1);
    assert_eq!(slice3, slice3);
    assert_eq!(slice2, slice2);
    assert_eq!(slice2, slice4);
    assert_eq!(slice4, slice2);
    assert_eq!(slice4, slice4);

    assert_ne!(slice1, slice2);
    assert_ne!(slice2, slice1);
    assert_ne!(slice3, slice4);
    assert_ne!(slice4, slice3);
    assert_ne!(slice1, slice4);
    assert_ne!(slice4, slice1);
    assert_ne!(slice2, slice3);
    assert_ne!(slice3, slice2);
  }

  #[test]
  fn test_iter_slices() {
    let two = Expr::from(2);
    let three = Expr::from(3);
    let slice1 = OrderedExprSlice::Borrowed { elems: &[Expr::from(2), Expr::from(3)] };
    let slice2 = OrderedExprSlice::Owned { elems: vec![&two, &three] };

    assert_eq!(
      slice1.iter().collect::<Vec<_>>(),
      vec![&Expr::from(2), &Expr::from(3)],
    );
    assert_eq!(
      slice2.iter().collect::<Vec<_>>(),
      vec![&Expr::from(2), &Expr::from(3)],
    );
  }

  #[test]
  fn test_factor_as_ordered_expr() {
    let expr = Expr::from(10);
    assert_eq!(OrderedExpr::from(&Factor::parse(expr.clone())), OrderedExpr::from(&expr));

    let expr = Expr::call("+", vec![Expr::var("x").unwrap(), Expr::from(10)]);
    assert_eq!(OrderedExpr::from(&Factor::parse(expr.clone())), OrderedExpr::from(&expr));

    let expr = Expr::call("^", vec![Expr::var("x").unwrap(), Expr::from(10)]);
    assert_eq!(OrderedExpr::from(&Factor::parse(expr.clone())), OrderedExpr::from(&expr));
  }

  #[test]
  fn test_factor_as_ordered_expr_with_factor_that_flattens_nested_exponents() {
    let expr = Expr::call("^", vec![
      Expr::call("^", vec![Expr::var("x").unwrap(), Expr::var("y").unwrap()]),
      Expr::from(10),
    ]);
    let result_expr = Expr::call("^", vec![
      Expr::var("x").unwrap(),
      Expr::call("*", vec![Expr::var("y").unwrap(), Expr::from(10)]),
    ]);
    assert_eq!(OrderedExpr::from(&Factor::parse(expr)), OrderedExpr::from(&result_expr));
  }

  #[test]
  fn test_factor_as_ordered_expr_with_factor_that_flattens_nested_exponents_arithmetically() {
    let expr = Expr::call("^", vec![
      Expr::call("^", vec![Expr::var("x").unwrap(), Expr::from(2)]),
      Expr::from(10),
    ]);
    let result_expr = Expr::call("^", vec![
      Expr::var("x").unwrap(),
      Expr::from(20),
    ]);
    assert_eq!(OrderedExpr::from(&Factor::parse(expr)), OrderedExpr::from(&result_expr));
  }
}
