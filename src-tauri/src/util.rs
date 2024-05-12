
//! Various utility functions.

use std::convert::Infallible;

pub fn unwrap_infallible<T>(res: Result<T, Infallible>) -> T {
  match res {
    Ok(res) => res,
    Err(_) => unreachable!(),
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn unwrap_infallible_unwraps() {
    let res = Ok(1);
    assert_eq!(unwrap_infallible(res), 1);
  }
}
