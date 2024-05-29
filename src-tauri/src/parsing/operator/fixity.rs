
//! Fixity declarations for operators.

use super::associativity::Associativity;
use super::precedence::Precedence;

use bitflags::bitflags;

/// An operator can be infix, prefix, postfix, or any combination
/// thereof. An operator will always be at least one of prefix,
/// postfix, or infix.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Fixity {
  as_prefix: Option<PrefixProperties>,
  as_infix: Option<InfixProperties>,
  as_postfix: Option<PostfixProperties>,
}

/// Unlike prefix and postfix operators, infix operators have both
/// associativity and precedence.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct InfixProperties {
  function_name: String,
  associativity: Associativity,
  precedence: Precedence,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PrefixProperties {
  function_name: String,
  precedence: Precedence,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PostfixProperties {
  function_name: String,
  precedence: Precedence,
}

bitflags! {
  pub struct FixityTypes: u8 {
    const PREFIX  = 0b0001;
    const INFIX   = 0b0010;
    const POSTFIX = 0b0100;
  }
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
  // allow: EmptyFixity is conceptually a Fixity, just with some
  // typechecks. It's intended to be used in a fluent builder style.
  #[allow(clippy::new_ret_no_self)]
  pub fn new() -> EmptyFixity {
    EmptyFixity {
      data: Fixity {
        as_prefix: None,
        as_infix: None,
        as_postfix: None,
      },
    }
  }

  pub fn with_prefix(mut self, function_name: impl Into<String>, p: Precedence) -> Fixity {
    let function_name = function_name.into();
    self.as_prefix = Some(PrefixProperties { function_name, precedence: p });
    self
  }

  pub fn with_infix(mut self, function_name: impl Into<String>, a: Associativity, p: Precedence) -> Fixity {
    let function_name = function_name.into();
    self.as_infix = Some(InfixProperties { function_name, associativity: a, precedence: p });
    self
  }

  pub fn with_postfix(mut self, function_name: impl Into<String>, p: Precedence) -> Fixity {
    let function_name = function_name.into();
    self.as_postfix = Some(PostfixProperties { function_name, precedence: p });
    self
  }

  pub fn as_prefix(&self) -> Option<&PrefixProperties> {
    self.as_prefix.as_ref()
  }

  pub fn as_infix(&self) -> Option<&InfixProperties> {
    self.as_infix.as_ref()
  }

  pub fn as_postfix(&self) -> Option<&PostfixProperties> {
    self.as_postfix.as_ref()
  }

  pub fn is_prefix(&self) -> bool {
    self.as_infix.is_some()
  }

  pub fn is_infix(&self) -> bool {
    self.as_infix.is_some()
  }

  pub fn is_postfix(&self) -> bool {
    self.as_postfix.is_some()
  }

  pub fn fixity_types(&self) -> FixityTypes {
    let mut t = FixityTypes::empty();
    if self.as_prefix.is_some() {
      t |= FixityTypes::PREFIX;
    }
    if self.as_infix.is_some() {
      t |= FixityTypes::INFIX;
    }
    if self.as_postfix.is_some() {
      t |= FixityTypes::POSTFIX;
    }
    t
  }
}

impl InfixProperties {
  pub fn function_name(&self) -> &str {
    &self.function_name
  }

  pub fn left_precedence(&self) -> Precedence {
    if self.associativity.is_left_assoc() {
      self.precedence
    } else {
      self.precedence.incremented()
    }
  }

  pub fn right_precedence(&self) -> Precedence {
    if self.associativity.is_right_assoc() {
      self.precedence
    } else {
      self.precedence.incremented()
    }
  }

  pub fn precedence(&self) -> Precedence {
    self.precedence
  }

  pub fn associativity(&self) -> Associativity {
    self.associativity
  }
}

impl PrefixProperties {
  pub fn function_name(&self) -> &str {
    &self.function_name
  }

  pub fn precedence(&self) -> Precedence {
    self.precedence
  }
}

impl PostfixProperties {
  pub fn function_name(&self) -> &str {
    &self.function_name
  }

  pub fn precedence(&self) -> Precedence {
    self.precedence
  }
}

impl EmptyFixity {
  pub fn with_prefix(self, function_name: impl Into<String>, p: Precedence) -> Fixity {
    self.data.with_prefix(function_name, p)
  }

  pub fn with_infix(self, function_name: impl Into<String>, a: Associativity, p: Precedence) -> Fixity {
    self.data.with_infix(function_name, a, p)
  }

  pub fn with_postfix(self, function_name: impl Into<String>, p: Precedence) -> Fixity {
    self.data.with_postfix(function_name, p)
  }
}
