
pub mod error;
pub mod shuffle;
mod structure;

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
