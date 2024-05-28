
//! Basic parsing language for the [basic language
//! mode](crate::display::basic::BasicLanguageMode].

use super::Expr;
use super::tokenizer::ExprTokenizer;
use crate::parsing::shunting_yard::ShuntingYardDriver;
use crate::parsing::operator::{Operator, OperatorTable};

use std::convert::Infallible;

#[derive(Clone, Debug)]
pub struct ExprParser<'a> {
  tokenizer: ExprTokenizer<'a>,
  operator_table: &'a OperatorTable,
}

#[derive(Clone, Debug, Default)]
#[non_exhaustive]
pub struct ExprShuntingYardDriver {}

impl<'a> ExprParser<'a> {
  pub fn new(operator_table: &'a OperatorTable) -> Self {
    Self {
      tokenizer: ExprTokenizer::new(operator_table),
      operator_table,
    }
  }

  pub fn tokenizer(&self) -> &ExprTokenizer<'a> {
    &self.tokenizer
  }
}

impl ExprShuntingYardDriver {
  pub fn new() -> Self {
    Self {}
  }
}

impl ShuntingYardDriver<Expr> for ExprShuntingYardDriver {
  type Output = Expr;
  type Error = Infallible;

  fn compile_scalar(&mut self, scalar: Expr) -> Result<Expr, Infallible> {
    Ok(scalar)
  }

  fn compile_bin_op(&mut self, left: Expr, oper: Operator, right: Expr) -> Result<Expr, Infallible> {
    Ok(Expr::call(oper.name(), vec![left, right]))
  }
}
