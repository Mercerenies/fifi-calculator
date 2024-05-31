
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
  use crate::expr::number::Number;

  pub fn stack_of(number_vec: Vec<i64>) -> Stack<Expr> {
    let expr_vec: Vec<_> = number_vec.into_iter().map(|n| {
      Expr::from(Number::from(n))
    }).collect();
    Stack::from(expr_vec)
  }
}
