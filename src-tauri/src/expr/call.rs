
use super::{Expr, TryFromExprError};

/// An expression which is guaranteed to be a functor call. This is
/// used as a prism target for some reflection operations in the
/// built-in function library.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct CallExpr {
  pub name: String,
  pub args: Vec<Expr>,
}

impl From<CallExpr> for Expr {
  fn from(c: CallExpr) -> Expr {
    Expr::Call(c.name, c.args)
  }
}

impl TryFrom<Expr> for CallExpr {
  type Error = TryFromExprError;

  fn try_from(e: Expr) -> Result<Self, Self::Error> {
    const TYPE_NAME: &str = "CallExpr";
    match e {
      Expr::Call(name, args) => Ok(CallExpr { name, args }),
      e => Err(TryFromExprError::new(TYPE_NAME, e)),
    }
  }
}
