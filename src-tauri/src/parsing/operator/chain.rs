
use super::{Operator, OperWithFixity};
use super::fixity::FixityType;
use crate::parsing::source::{Span, Spanned};
use crate::parsing::shunting_yard::{Token as ShuntingYardToken};
use crate::util::{count_prefix, count_suffix};

use thiserror::Error;

use std::error::{Error as StdError};
use std::fmt::{self, Display, Formatter};
use std::cmp::Ordering;

#[derive(Clone, Debug, Error)]
pub enum OperatorChainError<T> {
  #[error("{0}")]
  ChainParseError(#[from] ChainParseError),
  #[error("Adjacent terms not permitted: {0} and {1}")]
  AdjacentTermsNotPermitted(Spanned<T>, Spanned<T>),
}

#[derive(Clone, Debug)]
pub struct ChainParseError {
  failing_chain: Vec<Spanned<Operator>>,
}

#[derive(Clone, Debug)]
pub enum ChainToken<T> {
  Scalar(T, Span),
  Operator(Operator, Span),
}

/// Alternating sequence of operators, then a term, then operators,
/// then a term, etc. With the exception of the very first and very
/// last operator chain, all operator chains MUST be non-empty.
#[derive(Debug)]
struct AlternatingChainSeq<T> {
  chain_elements: Vec<AlternatingChainElem<T>>,
  operators_after: Vec<Spanned<Operator>>,
}

#[derive(Debug)]
struct AlternatingChainElem<T> {
  operators_before: Vec<Spanned<Operator>>,
  term: Spanned<T>,
}

impl<T> ChainToken<T> {
  pub fn span(&self) -> Span {
    match self {
      ChainToken::Scalar(_, span) => *span,
      ChainToken::Operator(_, span) => *span,
    }
  }
}

/// Given a chain of operators and expressions freely intermixed,
/// parses the operators and terms to produce tokens compatible with
/// the shunting yard algorithm.
///
/// Any operators before the first term must be prefix, any operators
/// after the last term must be postfix, and operators intermixed
/// between terms will be parsed with [`tag_operators_in_chain`].
/// Adjacent terms juxtaposed with no operators in between are not
/// permitted.
pub fn tag_chain_sequence<T>(tokens: Vec<ChainToken<T>>) -> Result<Vec<ShuntingYardToken<T>>, OperatorChainError<T>> {
  let chain_seq = parse_chain_sequence(tokens)?;
  let mut tokens = Vec::new();

  let mut first = true;
  for elem in chain_seq.chain_elements {
    if first {
      // Sequence of prefix operators.
      let prefix_ops = require_fixity_for_chain(elem.operators_before, FixityType::Prefix)?;
      tokens.extend(prefix_ops.into_iter().map(|op| ShuntingYardToken::operator(op.item, op.span)));
      tokens.push(ShuntingYardToken::scalar(elem.term.item, elem.term.span));
      first = false;
    } else {
      // Sequence between two terms.
      let ops = tag_operators_in_chain(elem.operators_before)?;
      tokens.extend(ops.into_iter().map(|op| ShuntingYardToken::operator(op.item, op.span)));
      tokens.push(ShuntingYardToken::scalar(elem.term.item, elem.term.span));
    }
  }

  // Sequence of trailing postfix operators.
  let postfix_ops = require_fixity_for_chain(chain_seq.operators_after, FixityType::Postfix)?;
  tokens.extend(postfix_ops.into_iter().map(|op| ShuntingYardToken::operator(op.item, op.span)));

  Ok(tokens)
}

fn parse_chain_sequence<T>(tokens: Vec<ChainToken<T>>) -> Result<AlternatingChainSeq<T>, OperatorChainError<T>> {
  let mut chain_elements: Vec<AlternatingChainElem<T>> = Vec::new();
  let mut current_seq: Vec<Spanned<Operator>> = Vec::new();
  for chain_token in tokens {
    match chain_token {
      ChainToken::Scalar(term, span) => {
        chain_elements.push(AlternatingChainElem { operators_before: current_seq, term: Spanned::new(term, span) });
        current_seq = Vec::new();
      }
      ChainToken::Operator(op, span) => {
        current_seq.push(Spanned::new(op, span));
      }
    }
  }

  // Validate the chain for consecutive terms.
  if let Some(problem_index) = find_consecutive_terms(&chain_elements) {
    let right_term = chain_elements.swap_remove(problem_index).term;
    let left_term = chain_elements.swap_remove(problem_index - 1).term;
    return Err(OperatorChainError::AdjacentTermsNotPermitted(left_term, right_term));
  }

  Ok(AlternatingChainSeq { chain_elements, operators_after: current_seq })
}

fn find_consecutive_terms<T>(seq: &[AlternatingChainElem<T>]) -> Option<usize> {
  for (i, elem) in seq.iter().enumerate() {
    // The first element is allowed to have no operators, since that
    // just implies a lack of prefix operators preceding the very
    // beginning of the expression.
    if i != 0 && elem.operators_before.is_empty() {
      return Some(i);
    }
  }
  None
}

/// Given a chain of one or more operators between two terms in the
/// term language, this function tags each operator with a
/// [`FixityType`]. Specifically, exactly one operator in the chain
/// will be tagged as infix, all operators before it will be tagged
/// postfix (and thus applied to the term(s) on the left), and all
/// operators after it will be tagged prefix (and thus applied to the
/// term(s) on the right).
///
/// If no such tagging exists, this function returns an appropriate
/// error. If the operators in this chain came from an operator table
/// which can be [parsed
/// unambiguously](super::table::OperatorTable::check_for_ambiguities),
/// then the returned tagging (if one exists) is guaranteed to be
/// unique. If the table contains ambiguities, then one valid tagging
/// will be returned, but no guarantees are made as to which one.
fn tag_operators_in_chain(
  operator_chain: Vec<Spanned<Operator>>,
) -> Result<Vec<Spanned<OperWithFixity>>, ChainParseError> {
  // Identify the longest prefix of our chain which consists of
  // postfix-compatible operators. Then identify the longest suffix of
  // our chain which consists of prefix-compatible operators.
  let initial_postfix_len = count_prefix(operator_chain.iter(), |op| op.item.fixity().is_postfix());
  let final_prefix_len = count_suffix(operator_chain.iter(), |op| op.item.fixity().is_prefix());
  let begin_pivot = (operator_chain.len() - final_prefix_len).saturating_sub(1);
  let end_pivot = (initial_postfix_len + 1).min(operator_chain.len());
  // The indices for `pivot` are our valid choices for the infix
  // operator in the middle. Find the first one that's
  // infix-compatible. If we can't find one, it's an error.
  for pivot in begin_pivot..end_pivot {
    if operator_chain[pivot].item.fixity().is_infix() {
      // We found an infix operator. Return the tagged chain.
      let tagged_chain = operator_chain.into_iter().enumerate().map(|(j, op)| {
        let target_fixity = match j.cmp(&pivot) {
          Ordering::Less => FixityType::Postfix,
          Ordering::Equal => FixityType::Infix,
          Ordering::Greater => FixityType::Prefix,
        };
        op.map(|op| OperWithFixity::new(op, target_fixity))
      }).collect();
      return Ok(tagged_chain);
    }
  }
  Err(ChainParseError { failing_chain: operator_chain })
}

/// Converts a sequence of [`Operator`] into a sequence of
/// [`OperWithFixity`] with the chosen fixity. If any operator does
/// not support the given fixity, produces an error.
fn require_fixity_for_chain(
  operator_chain: Vec<Spanned<Operator>>,
  fixity: FixityType,
) -> Result<Vec<Spanned<OperWithFixity>>, ChainParseError> {
  for op in &operator_chain {
    if !op.item.fixity().supports(fixity) {
      return Err(ChainParseError { failing_chain: operator_chain });
    }
  }
  Ok(operator_chain.into_iter().map(|op| {
    // safety: We already checked that all of the operators were good
    // for this fixity.
    op.map(|op| OperWithFixity::new(op, fixity))
  }).collect())
}

impl Display for ChainParseError {
  fn fmt(&self, f: &mut Formatter) -> fmt::Result {
    let operators = self.failing_chain.iter().map(|op| op.to_string()).collect::<Vec<_>>().join(" ");
    write!(f, "Failed to parse operator chain: {}", operators)
  }
}

impl StdError for ChainParseError {}

#[cfg(test)]
mod tests {
  use super::*;
  use crate::parsing::operator::{Fixity, Associativity, Precedence};

  fn spanned<T>(t: T) -> Spanned<T> {
    Spanned::new(t, Span::default())
  }

  fn infix(name: &str) -> Operator {
    Operator::new(
      name.to_owned(),
      Fixity::new()
        .with_infix(name.to_owned(), Associativity::LEFT, Precedence::new(0)),
    )
  }

  fn prefix(name: &str) -> Operator {
    Operator::new(
      name.to_owned(),
      Fixity::new()
        .with_prefix(name.to_owned(), Precedence::new(0)),
    )
  }

  fn postfix(name: &str) -> Operator {
    Operator::new(
      name.to_owned(),
      Fixity::new()
        .with_postfix(name.to_owned(), Precedence::new(0)),
    )
  }

  fn infix_prefix(name: &str) -> Operator {
    Operator::new(
      name.to_owned(),
      Fixity::new()
        .with_infix(name.to_owned(), Associativity::LEFT, Precedence::new(0))
        .with_prefix(name.to_owned(), Precedence::new(0)),
    )
  }

  fn infix_postfix(name: &str) -> Operator {
    Operator::new(
      name.to_owned(),
      Fixity::new()
        .with_infix(name.to_owned(), Associativity::LEFT, Precedence::new(0))
        .with_postfix(name.to_owned(), Precedence::new(0)),
    )
  }


  fn prefix_postfix(name: &str) -> Operator {
    Operator::new(
      name.to_owned(),
      Fixity::new()
        .with_postfix(name.to_owned(), Precedence::new(0))
        .with_prefix(name.to_owned(), Precedence::new(0)),
    )
  }

  fn infix_prefix_postfix(name: &str) -> Operator {
    Operator::new(
      name.to_owned(),
      Fixity::new()
        .with_infix(name.to_owned(), Associativity::LEFT, Precedence::new(0))
        .with_postfix(name.to_owned(), Precedence::new(0))
        .with_prefix(name.to_owned(), Precedence::new(0)),
    )
  }

  #[test]
  fn test_require_fixity_prefix() {
    let ops = vec![
      spanned(infix_prefix_postfix("a")),
      spanned(prefix_postfix("b")),
      spanned(prefix("c")),
      spanned(infix_prefix("d")),
    ];
    let result = require_fixity_for_chain(ops, FixityType::Prefix).unwrap();
    assert!(result.iter().all(|op| op.item.fixity_type() == FixityType::Prefix));
    let result = result.into_iter().map(|op| op.item.into_operator().operator_name().to_owned()).collect::<Vec<_>>();
    assert_eq!(result, vec!["a", "b", "c", "d"]);
  }

  #[test]
  fn test_require_fixity_postfix() {
    let ops = vec![
      spanned(infix_prefix_postfix("a")),
      spanned(prefix_postfix("b")),
      spanned(postfix("c")),
      spanned(infix_postfix("d")),
    ];
    let result = require_fixity_for_chain(ops, FixityType::Postfix).unwrap();
    assert!(result.iter().all(|op| op.item.fixity_type() == FixityType::Postfix));
    let result = result.into_iter().map(|op| op.item.into_operator().operator_name().to_owned()).collect::<Vec<_>>();
    assert_eq!(result, vec!["a", "b", "c", "d"]);
  }

  #[test]
  fn test_require_fixity_infix() {
    let ops = vec![
      spanned(infix_prefix_postfix("a")),
      spanned(infix_prefix("b")),
      spanned(infix("c")),
      spanned(infix_postfix("d")),
    ];
    let result = require_fixity_for_chain(ops, FixityType::Infix).unwrap();
    assert!(result.iter().all(|op| op.item.fixity_type() == FixityType::Infix));
    let result = result.into_iter().map(|op| op.item.into_operator().operator_name().to_owned()).collect::<Vec<_>>();
    assert_eq!(result, vec!["a", "b", "c", "d"]);
  }

  #[test]
  fn test_require_fixity_invalid() {
    let ops = vec![
      spanned(infix_prefix_postfix("a")),
      spanned(infix_prefix("b")),
      spanned(infix_postfix("c")),
    ];
    let err = require_fixity_for_chain(ops.clone(), FixityType::Prefix).unwrap_err();
    assert_eq!(err.failing_chain, ops);
  }

  #[test]
  fn test_tag_operators_in_chain_simple_infix() {
    let ops = vec![
      spanned(infix("a")),
    ];
    let result = tag_operators_in_chain(ops).unwrap();
    assert_eq!(
      result,
      vec![
        spanned(OperWithFixity::infix(infix("a"))),
      ],
    );
  }

  #[test]
  fn test_tag_operators_in_chain_infix_with_postfixes() {
    let ops = vec![
      spanned(postfix("post1")),
      spanned(infix_prefix_postfix("post2")),
      spanned(infix_postfix("post3")),
      spanned(infix("a")),
    ];
    let result = tag_operators_in_chain(ops).unwrap();
    assert_eq!(
      result,
      vec![
        spanned(OperWithFixity::postfix(postfix("post1"))),
        spanned(OperWithFixity::postfix(infix_prefix_postfix("post2"))),
        spanned(OperWithFixity::postfix(infix_postfix("post3"))),
        spanned(OperWithFixity::infix(infix("a"))),
      ],
    );
  }

  #[test]
  fn test_tag_operators_in_chain_infix_with_prefixes() {
    let ops = vec![
      spanned(infix("a")),
      spanned(infix_prefix_postfix("pre1")),
      spanned(infix_prefix("pre2")),
      spanned(prefix("pre3")),
    ];
    let result = tag_operators_in_chain(ops).unwrap();
    assert_eq!(
      result,
      vec![
        spanned(OperWithFixity::infix(infix("a"))),
        spanned(OperWithFixity::prefix(infix_prefix_postfix("pre1"))),
        spanned(OperWithFixity::prefix(infix_prefix("pre2"))),
        spanned(OperWithFixity::prefix(prefix("pre3"))),
      ],
    );
  }

  #[test]
  fn test_tag_operators_in_chain_infix_with_all() {
    let ops = vec![
      spanned(infix_prefix_postfix("post1")),
      spanned(infix("a")),
      spanned(infix_prefix_postfix("pre1")),
    ];
    let result = tag_operators_in_chain(ops).unwrap();
    assert_eq!(
      result,
      vec![
        spanned(OperWithFixity::postfix(infix_prefix_postfix("post1"))),
        spanned(OperWithFixity::infix(infix("a"))),
        spanned(OperWithFixity::prefix(infix_prefix_postfix("pre1"))),
      ],
    );
  }

  #[test]
  fn test_tag_operators_in_chain_invalid_input() {
    let ops = vec![
      spanned(infix_prefix_postfix("x")),
      spanned(infix("x")),
      spanned(infix_postfix("x")),
    ];
    tag_operators_in_chain(ops).unwrap_err();

    let ops = vec![
      spanned(prefix("x")),
      spanned(infix_prefix_postfix("x")),
      spanned(infix_prefix_postfix("x")),
    ];
    tag_operators_in_chain(ops).unwrap_err();

    let ops = vec![
      spanned(infix_prefix_postfix("x")),
      spanned(prefix("x")),
      spanned(infix_prefix_postfix("x")),
      spanned(infix_prefix_postfix("x")),
      spanned(infix_prefix_postfix("x")),
      spanned(infix_prefix_postfix("x")),
      spanned(infix_postfix("x")),
    ];
    tag_operators_in_chain(ops).unwrap_err();

    let ops = vec![
      spanned(prefix_postfix("x")),
    ];
    tag_operators_in_chain(ops).unwrap_err();

    let ops = vec![];
    tag_operators_in_chain(ops).unwrap_err();
  }

  ///// more tests
}
