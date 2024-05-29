
//! Fixity declarations for operators.

use super::associativity::Associativity;
use super::precedence::Precedence;

/// An operator can be infix, prefix, postfix, or any combination
/// thereof. An operator will always be at least one of prefix,
/// postfix, or infix.
#[derive(Clone, Debug, PartialEq, Eq, Copy)]
pub struct Fixity {
  as_prefix: Option<Precedence>,
  as_infix: Option<InfixProperties>,
  as_postfix: Option<Precedence>,
}

/// Unlike prefix and postfix operators, infix operators have both
/// associativity and precedence.
#[derive(Clone, Debug, PartialEq, Eq, Copy)]
pub struct InfixProperties {
  assoc: Associativity,
  prec: Precedence,
}

/// The type of an "empty" fixity structure. This is an intermediate
/// type which is only used during building of a [`Fixity`]. This type
/// is used to guarantee the precondition that a `Fixity` structure
/// always has at least one valid fixity type (prefix, infix, or
/// postfix).
#[derive(Debug)]
pub struct EmptyFixity {
  data: Fixity,
}

impl Fixity {
  pub fn new() -> EmptyFixity {
    EmptyFixity {
      data: Fixity {
        as_prefix: None,
        as_infix: None,
        as_postfix: None,
      },
    }
  }

  pub fn with_prefix(mut self, p: Precedence) -> Fixity {
    self.as_prefix = Some(p);
    self
  }

  pub fn with_infix(mut self, a: Associativity, p: Precedence) -> Fixity {
    self.as_infix = Some(InfixProperties { assoc: a, prec: p });
    self
  }

  pub fn with_postfix(mut self, p: Precedence) -> Fixity {
    self.as_postfix = Some(p);
    self
  }

  pub fn prefix_prec(&self) -> Option<Precedence> {
    self.as_prefix
  }

  pub fn as_infix(&self) -> Option<InfixProperties> {
    self.as_infix
  }

  pub fn infix_prec(&self) -> Option<Precedence> {
    self.as_infix.map(|i| i.prec)
  }

  pub fn infix_assoc(&self) -> Option<Associativity> {
    self.as_infix.map(|i| i.assoc)
  }

  pub fn postfix_prec(&self) -> Option<Precedence> {
    self.as_postfix
  }
}

impl EmptyFixity {
  pub fn with_prefix(self, p: Precedence) -> Fixity {
    self.data.with_prefix(p)
  }

  pub fn with_infix(self, a: Associativity, p: Precedence) -> Fixity {
    self.data.with_infix(a, p)
  }

  pub fn with_postfix(self, p: Precedence) -> Fixity {
    self.data.with_postfix(p)
  }
}
