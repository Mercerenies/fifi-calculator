
use super::{Operator, OperWithFixity};
use super::fixity::FixityType;
use crate::parsing::source::Span;
use crate::util::{count_prefix, count_suffix};

use std::error::{Error as StdError};
use std::fmt::{self, Display, Formatter};
use std::cmp::Ordering;

#[derive(Clone, Debug)]
pub struct OperatorChainError {
  implementation: OperatorChainErrorImpl,
}

#[derive(Clone, Debug)]
enum OperatorChainErrorImpl {
  CouldNotParseChain {
    failing_chain: Vec<Operator>,
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
pub fn tag_operators_in_chain(operator_chain: Vec<Operator>) -> Result<Vec<OperWithFixity>, OperatorChainError> {
  // Identify the longest prefix of our chain which consists of
  // postfix-compatible operators. Then identify the longest suffix of
  // our chain which consists of prefix-compatible operators.
  let first_non_postfix = count_prefix(operator_chain.iter(), |op| op.fixity().is_postfix());
  let last_non_prefix = operator_chain.len() - count_suffix(operator_chain.iter(), |op| op.fixity().is_prefix()) - 1;
  // The indices for `pivot` are our valid choices for the infix
  // operator in the middle. Find the first one that's
  // infix-compatible. If we can't find one, it's an error.
  for pivot in (last_non_prefix + 1)..first_non_postfix {
    if operator_chain[pivot].fixity().is_infix() {
      // We found an infix operator. Return the tagged chain.
      let tagged_chain = operator_chain.into_iter().enumerate().map(|(j, op)| {
        let target_fixity = match j.cmp(&pivot) {
          Ordering::Less => FixityType::Postfix,
          Ordering::Equal => FixityType::Infix,
          Ordering::Greater => FixityType::Prefix,
        };
        OperWithFixity::new(op, target_fixity)
      }).collect();
      return Ok(tagged_chain);
    }
  }
  Err(OperatorChainError::could_not_parse_chain(operator_chain))
}

/// Converts a sequence of [`Operator`] into a sequence of
/// [`OperWithFixity`] with the chosen fixity. If any operator does
/// not support the given fixity, produces an error.
pub fn require_fixity_for_chain(
  operator_chain: Vec<Operator>,
  fixity: FixityType,
) -> Result<Vec<OperWithFixity>, OperatorChainError> {
  for op in &operator_chain {
    if !op.fixity().supports(fixity) {
      return Err(OperatorChainError::could_not_parse_chain(operator_chain));
    }
  }
  Ok(operator_chain.into_iter().map(|op| {
    // safety: We already checked that all of the operators were good
    // for this fixity.
    OperWithFixity::new(op, fixity)
  }).collect())
}

impl OperatorChainError {
  fn could_not_parse_chain(chain: Vec<Operator>) -> OperatorChainError {
    OperatorChainError {
      implementation: OperatorChainErrorImpl::CouldNotParseChain { failing_chain: chain },
    }
  }
}

impl Display for OperatorChainError {
  fn fmt(&self, f: &mut Formatter) -> fmt::Result {
    self.implementation.fmt(f)
  }
}

impl StdError for OperatorChainError {}

impl Display for OperatorChainErrorImpl {
  fn fmt(&self, f: &mut Formatter) -> fmt::Result {
    match self {
      OperatorChainErrorImpl::CouldNotParseChain { failing_chain } => {
        let operators = failing_chain.iter().map(|op| op.to_string()).collect::<Vec<_>>().join(" ");
        write!(f, "Failed to parse operator chain: {}", operators)
      }
    }
  }
}

impl StdError for OperatorChainErrorImpl {}
