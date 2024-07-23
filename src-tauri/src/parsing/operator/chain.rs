
use super::{Operator, TaggedOperator};
use super::fixity::FixityType;
use crate::parsing::source::Spanned;
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

/// A token, for the purposes of operator chain resolution, is either
/// a scalar value or an operator.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Token<T> {
  Scalar(T),
  Operator(Operator),
}

/// A token, tagged with operator fixity information.
#[derive(Clone, Debug)]
pub enum TaggedToken<T> {
  Scalar(T),
  Operator(TaggedOperator),
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

/// Given a chain of operators and expressions freely intermixed,
/// parses the operators and terms to produce tokens compatible with
/// the shunting yard algorithm.
///
/// Any operators before the first term must be prefix, any operators
/// after the last term must be postfix, and operators intermixed
/// between terms will be parsed with [`tag_operators_in_chain`].
/// Adjacent terms juxtaposed with no operators in between are not
/// permitted.
pub fn tag_chain_sequence<T>(
  tokens: Vec<Spanned<Token<T>>>,
) -> Result<Vec<Spanned<TaggedToken<T>>>, OperatorChainError<T>> {
  let chain_seq = parse_chain_sequence(tokens)?;
  let mut tokens = Vec::new();

  let mut first = true;
  for elem in chain_seq.chain_elements {
    if first {
      // Sequence of prefix operators.
      let prefix_ops = require_fixity_for_chain(elem.operators_before, FixityType::Prefix)?;
      tokens.extend(prefix_ops.into_iter().map(|op| op.map(TaggedToken::Operator)));
      tokens.push(elem.term.map(TaggedToken::Scalar));
      first = false;
    } else {
      // Sequence between two terms.
      let ops = tag_operators_in_chain(elem.operators_before)?;
      tokens.extend(ops.into_iter().map(|op| op.map(TaggedToken::Operator)));
      tokens.push(elem.term.map(TaggedToken::Scalar));
    }
  }

  // Sequence of trailing postfix operators.
  let postfix_ops = require_fixity_for_chain(chain_seq.operators_after, FixityType::Postfix)?;
  tokens.extend(postfix_ops.into_iter().map(|op| op.map(TaggedToken::Operator)));

  Ok(tokens)
}

fn parse_chain_sequence<T>(
  tokens: Vec<Spanned<Token<T>>>,
) -> Result<AlternatingChainSeq<T>, OperatorChainError<T>> {
  let mut chain_elements: Vec<AlternatingChainElem<T>> = Vec::new();
  let mut current_seq: Vec<Spanned<Operator>> = Vec::new();
  for chain_token in tokens {
    match chain_token.item {
      Token::Scalar(term) => {
        chain_elements.push(AlternatingChainElem {
          operators_before: current_seq,
          term: Spanned::new(term, chain_token.span),
        });
        current_seq = Vec::new();
      }
      Token::Operator(op) => {
        current_seq.push(Spanned::new(op, chain_token.span));
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

/// Scans the vector `tokens` for any place where there are two terms
/// adjacent to one another, and inserts the given infix operator in
/// such places.
pub fn insert_juxtaposition_operator<T>(
  tokens: &mut Vec<Spanned<Token<T>>>,
  operator: Operator,
) {
  let mut i = 0;
  while i < tokens.len() - 1 {
    if tokens[i].item.is_scalar() && tokens[i + 1].item.is_scalar() {
      let span = tokens[i + 1].span;
      tokens.insert(i + 1, Spanned::new(Token::Operator(operator.clone()), span));
      i += 2;
    } else {
      i += 1;
    }
  }
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
pub fn tag_operators_in_chain(
  operator_chain: Vec<Spanned<Operator>>,
) -> Result<Vec<Spanned<TaggedOperator>>, ChainParseError> {
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
        op.map(|op| TaggedOperator::new(op, target_fixity))
      }).collect();
      return Ok(tagged_chain);
    }
  }
  Err(ChainParseError { failing_chain: operator_chain })
}

/// Converts a sequence of [`Operator`] into a sequence of
/// [`TaggedOperator`] with the chosen fixity. If any operator does
/// not support the given fixity, produces an error.
fn require_fixity_for_chain(
  operator_chain: Vec<Spanned<Operator>>,
  fixity: FixityType,
) -> Result<Vec<Spanned<TaggedOperator>>, ChainParseError> {
  for op in &operator_chain {
    if !op.item.fixity().supports(fixity) {
      return Err(ChainParseError { failing_chain: operator_chain });
    }
  }
  Ok(operator_chain.into_iter().map(|op| {
    // safety: We already checked that all of the operators were good
    // for this fixity.
    op.map(|op| TaggedOperator::new(op, fixity))
  }).collect())
}

impl<T> Token<T> {
  pub fn is_scalar(&self) -> bool {
    matches!(self, Token::Scalar(_))
  }

  pub fn is_operator(&self) -> bool {
    matches!(self, Token::Operator(_))
  }
}

impl<T> TaggedToken<T> {
  pub fn infix_operator(operator: Operator) -> Self {
    TaggedToken::Operator(TaggedOperator::infix(operator))
  }
  pub fn postfix_operator(operator: Operator) -> Self {
    TaggedToken::Operator(TaggedOperator::postfix(operator))
  }
  pub fn prefix_operator(operator: Operator) -> Self {
    TaggedToken::Operator(TaggedOperator::prefix(operator))
  }
}

impl Display for ChainParseError {
  fn fmt(&self, f: &mut Formatter) -> fmt::Result {
    let operators = self.failing_chain.iter().map(|op| op.to_string()).collect::<Vec<_>>().join(" ");
    write!(f, "Failed to parse operator chain: {}", operators)
  }
}

impl StdError for ChainParseError {}

impl<T: Display> Display for Token<T> {
  fn fmt(&self, f: &mut Formatter) -> fmt::Result {
    match self {
      Token::Scalar(s) => write!(f, "{}", s),
      Token::Operator(o) => write!(f, "{}", o),
    }
  }
}

impl<T: Display> Display for TaggedToken<T> {
  fn fmt(&self, f: &mut Formatter) -> fmt::Result {
    match self {
      TaggedToken::Scalar(s) => write!(f, "{}", s),
      TaggedToken::Operator(o) => write!(f, "{}", o),
    }
  }
}

impl<T> From<TaggedOperator> for TaggedToken<T> {
  fn from(op: TaggedOperator) -> Self {
    TaggedToken::Operator(op)
  }
}

impl<T> From<Operator> for Token<T> {
  fn from(op: Operator) -> Self {
    Token::Operator(op)
  }
}

#[cfg(test)]
mod tests {
  use super::*;
  use crate::parsing::source::Span;
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
        spanned(TaggedOperator::infix(infix("a"))),
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
        spanned(TaggedOperator::postfix(postfix("post1"))),
        spanned(TaggedOperator::postfix(infix_prefix_postfix("post2"))),
        spanned(TaggedOperator::postfix(infix_postfix("post3"))),
        spanned(TaggedOperator::infix(infix("a"))),
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
        spanned(TaggedOperator::infix(infix("a"))),
        spanned(TaggedOperator::prefix(infix_prefix_postfix("pre1"))),
        spanned(TaggedOperator::prefix(infix_prefix("pre2"))),
        spanned(TaggedOperator::prefix(prefix("pre3"))),
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
        spanned(TaggedOperator::postfix(infix_prefix_postfix("post1"))),
        spanned(TaggedOperator::infix(infix("a"))),
        spanned(TaggedOperator::prefix(infix_prefix_postfix("pre1"))),
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

  #[test]
  fn test_insert_juxtaposition_operator() {
    let juxtaposition_operator = infix("yy");
    let mut chain = vec![
      spanned(Token::Scalar(1)),
      spanned(Token::Operator(infix("xx"))),
      spanned(Token::Scalar(2)),
      spanned(Token::Scalar(3)),
      spanned(Token::Scalar(4)),
      spanned(Token::Operator(infix("xx"))),
      spanned(Token::Scalar(5)),
    ];
    insert_juxtaposition_operator(&mut chain, juxtaposition_operator);
    assert_eq!(
      chain,
      vec![
        spanned(Token::Scalar(1)),
        spanned(Token::Operator(infix("xx"))),
        spanned(Token::Scalar(2)),
        spanned(Token::Operator(infix("yy"))),
        spanned(Token::Scalar(3)),
        spanned(Token::Operator(infix("yy"))),
        spanned(Token::Scalar(4)),
        spanned(Token::Operator(infix("xx"))),
        spanned(Token::Scalar(5)),
      ]
    );
  }
}
