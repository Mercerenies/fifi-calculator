
use crate::expr::number::Number;

/// A basic expression type, without any operator precedence
/// semantics.
#[derive(Clone, Debug)]
pub enum BasicExpr {
  Scalar(Number),
  OperatorChain(Box<BasicExpr>, Vec<OperatorApp>),
}

#[derive(Clone, Debug)]
pub struct OperatorApp {
  pub operator: String,
  pub right_hand_side: BasicExpr,
}
