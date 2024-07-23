
//! Queries operate by asking for certain Boolean information from
//! elements on the main value stack.

use crate::expr::Expr;
use crate::expr::number::Number;
use crate::expr::algebra::term::TermParser;
use crate::expr::units::parse_composite_unit_expr;
use crate::stack::StackError;
use crate::stack::base::RandomAccessStackLike;
use crate::units::parsing::UnitParser;
use crate::units::tagged::TemperatureTagged;

use thiserror::Error;

use serde::{Serialize, Deserialize};

/// A query targeting a specific stack position. As with most stack
/// functions, nonnegative indices count from the top of the stack,
/// while negative indices count from the bottom. So 0 always refers
/// to the top of the stack and -1 always refers to the bottom.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
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
  /// Query that checks whether the target expression contains units
  /// whose dimension is temperature.
  HasBasicTemperatureUnits,
}

#[derive(Clone, Debug, Error, PartialEq, Eq)]
#[non_exhaustive]
pub enum QueryError {
  #[error("{0}")]
  StackError(#[from] StackError),
}

#[derive(Clone)]
pub struct QueryContext<'a, 'b> {
  pub units_parser: &'a dyn UnitParser<Number>,
  pub term_parser: &'b TermParser,
}

pub fn run_query<S>(query: &Query, context: &QueryContext, stack: &S) -> Result<bool, QueryError>
where S: RandomAccessStackLike<Elem = Expr> {
  let stack_elem = stack.get(query.stack_index)?;
  match query.query_type {
    QueryType::HasUnits => {
      Ok(has_any_units(context, stack_elem.to_owned())) // TODO: Excessive cloning?
    }
    QueryType::HasBasicTemperatureUnits => {
      Ok(has_basic_temperature_units(context, stack_elem.to_owned())) // TODO: Excessive cloning?
    }
  }
}

pub fn has_any_units(context: &QueryContext, expr: Expr) -> bool {
  let tagged_expr = parse_composite_unit_expr(context.units_parser, context.term_parser, expr);
  !tagged_expr.unit.is_empty()
}

pub fn has_basic_temperature_units(context: &QueryContext, expr: Expr) -> bool {
  let tagged_expr = parse_composite_unit_expr(context.units_parser, context.term_parser, expr);
  TemperatureTagged::try_from(tagged_expr).is_ok()
}

#[cfg(test)]
mod tests {
  use super::*;
  use crate::units::parsing::default_parser;
  use crate::stack::test_utils::stack_of;

  fn var(x: &str) -> Expr {
    Expr::var(x).unwrap()
  }

  #[test]
  fn test_has_any_units() {
    let unit_parser = default_parser();
    let term_parser = TermParser::new();
    let context = QueryContext { units_parser: &unit_parser, term_parser: &term_parser };
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

  #[test]
  fn test_run_query_on_stack() {
    let unit_parser = default_parser();
    let term_parser = TermParser::new();
    let context = QueryContext { units_parser: &unit_parser, term_parser: &term_parser };
    let stack = stack_of(vec![Expr::from(100), Expr::from(200), var("km")]);
    assert_eq!(
      run_query(&Query { stack_index: 0, query_type: QueryType::HasUnits }, &context, &stack),
      Ok(true),
    );
    assert_eq!(
      run_query(&Query { stack_index: 1, query_type: QueryType::HasUnits }, &context, &stack),
      Ok(false),
    );
    assert_eq!(
      run_query(&Query { stack_index: 2, query_type: QueryType::HasUnits }, &context, &stack),
      Ok(false),
    );
    assert_eq!(
      run_query(&Query { stack_index: 3, query_type: QueryType::HasUnits }, &context, &stack),
      Err(QueryError::StackError(StackError::NotEnoughElements { expected: 4, actual: 3 })),
    );
    assert_eq!(
      run_query(&Query { stack_index: -1, query_type: QueryType::HasUnits }, &context, &stack),
      Ok(false),
    );
    assert_eq!(
      run_query(&Query { stack_index: -2, query_type: QueryType::HasUnits }, &context, &stack),
      Ok(false),
    );
    assert_eq!(
      run_query(&Query { stack_index: -3, query_type: QueryType::HasUnits }, &context, &stack),
      Ok(true),
    );
    assert_eq!(
      run_query(&Query { stack_index: -4, query_type: QueryType::HasUnits }, &context, &stack),
      Err(QueryError::StackError(StackError::NotEnoughElements { expected: 4, actual: 3 })),
    );
  }
}
