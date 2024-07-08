
//! Queries operate by asking for certain Boolean information from
//! elements on the main value stack.

use crate::expr::Expr;
use crate::expr::number::Number;
use crate::expr::units::parse_composite_unit_expr;
use crate::stack::StackError;
use crate::stack::base::RandomAccessStackLike;
use crate::units::parsing::UnitParser;

use thiserror::Error;

use serde::{Serialize, Deserialize};

/// A query targeting a specific stack position. As with most stack
/// functions, nonnegative indices count from the top of the stack,
/// while negative indices count from the bottom. So 0 always refers
/// to the top of the stack and -1 always refers to the bottom.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Query {
  pub stack_index: i64,
  pub query_type: QueryType,
}

/// Types of queries that can be asked of an expression.
#[derive(Debug, Copy, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum QueryType {
  /// Query that checks whether the target expression contains any
  /// units, as defined by the context's unit parser. Delegates to
  /// [`has_any_units`].
  HasUnits,
}

#[derive(Clone, Debug, Error)]
#[non_exhaustive]
pub enum QueryError {
  #[error("{0}")]
  StackError(#[from] StackError),
}

#[derive(Clone)]
pub struct QueryContext<'a> {
  pub units_parser: &'a dyn UnitParser<Number>,
}

pub fn run_query<S>(query: &Query, context: &QueryContext, stack: &S) -> Result<bool, QueryError>
where S: RandomAccessStackLike<Elem = Expr> {
  let stack_elem = stack.get(query.stack_index)?;
  match query.query_type {
    QueryType::HasUnits => {
      Ok(has_any_units(context, stack_elem.to_owned())) // TODO: Excessive cloning?
    }
  }
}

pub fn has_any_units(context: &QueryContext, expr: Expr) -> bool {
  let tagged_expr = parse_composite_unit_expr(context.units_parser, expr);
  !tagged_expr.unit.is_empty()
}

#[cfg(test)]
mod tests {
  use super::*;
  use crate::units::parsing::default_parser;

  fn var(x: &str) -> Expr {
    Expr::var(x).unwrap()
  }

  #[test]
  fn test_has_any_units() {
    let parser = default_parser();
    let context = QueryContext { units_parser: &parser };
    assert!(has_any_units(&context, var("m")));
    assert!(has_any_units(&context, var("km")));
    assert!(!has_any_units(&context, var("eggsalad")));
    assert!(has_any_units(&context,
                          Expr::call("*", vec![var("km"), Expr::call("^", vec![var("sec"), Expr::from(3)])])));
    assert!(has_any_units(&context,
                          Expr::call("/", vec![var("km"), Expr::call("^", vec![var("sec"), Expr::from(3)])])));
    assert!(has_any_units(&context,
                          Expr::call("*", vec![var("km"), Expr::call("^", vec![var("sec"), Expr::from(-10)])])));
    assert!(has_any_units(&context,
                          Expr::call("*", vec![var("km"), Expr::call("^", vec![var("sec"), Expr::from(0)])])));
    assert!(!has_any_units(&context,
                           Expr::call("+", vec![var("km"), var("cm")])));
  }
}
