
pub mod base;
mod delegate;
mod error;
pub mod keepable;
mod structure;

pub use delegate::{DelegatingStack, StackDelegate};
pub use error::StackError;
pub use structure::Stack;

#[cfg(test)]
pub(crate) mod test_utils {
  use super::*;
  use crate::expr::Expr;

  pub fn stack_of(vec: Vec<impl Into<Expr>>) -> Stack<Expr> {
    let expr_vec: Vec<_> = vec.into_iter().map(|n| n.into()).collect();
    Stack::from(expr_vec)
  }
}
