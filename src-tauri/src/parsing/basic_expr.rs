
use crate::expr::number::Number;
use super::source::Span;

/// A basic expression type, without any operator precedence
/// semantics.
#[derive(Clone, Debug)]
pub enum BasicExpr {
  Scalar(Number),
  OperatorChain(Box<(BasicExpr, Span)>, Vec<OperatorApp>),
}

#[derive(Clone, Debug)]
pub struct OperatorApp {
  pub operator: (String, Span),
  pub right: (BasicExpr, Span),
}
