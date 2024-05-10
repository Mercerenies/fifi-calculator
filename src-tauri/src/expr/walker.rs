
//! Utility functions for walking an expression tree.

use super::Expr;
use crate::util::unwrap_infallible;

pub fn postorder_walk<E, F>(expr: Expr, mut f: F) -> Result<Expr, E>
where F: FnMut(Expr) -> Result<Expr, E> {
  postorder_walk_impl(expr, &mut f)
}

pub fn postorder_walk_ok<F>(expr: Expr, mut f: F) -> Expr
where F: FnMut(Expr) -> Expr {
  let f_err = |expr| Ok(f(expr));
  unwrap_infallible(
    postorder_walk(expr, f_err)
  )
}

fn postorder_walk_impl<E, F>(expr: Expr, f: &mut F) -> Result<Expr, E>
where F: FnMut(Expr) -> Result<Expr, E> {
  let expr = match expr {
    Expr::Atom(atom) => {
      Expr::Atom(atom)
    }
    Expr::Call(function_name, args) => {
      let args = args.into_iter().map(|x| postorder_walk_impl(x, f)).collect::<Result<Vec<_>, _>>()?;
      Expr::Call(function_name, args)
    }
  };
  f(expr)
}
