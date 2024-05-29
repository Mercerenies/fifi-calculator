
/// The associativity of an infix operator.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Associativity {
  left_assoc: bool,
  right_assoc: bool,
}

impl Associativity {
  /// Indicates an operator which associates to the left.
  pub const LEFT: Associativity = Associativity {
    left_assoc: true,
    right_assoc: false,
  };
  /// Indicates an operator which associate to the right.
  pub const RIGHT: Associativity = Associativity {
    left_assoc: false,
    right_assoc: true,
  };
  /// Indicates a non-associative operator, which always requires
  /// parentheses for nested applications of itself.
  pub const NONE: Associativity = Associativity {
    left_assoc: false,
    right_assoc: false,
  };
  /// Indicates an associative operator for which the order of
  /// evaluation doesn't affect the result.
  pub const FULL: Associativity = Associativity {
    left_assoc: true,
    right_assoc: true,
  };
  pub const fn is_left_assoc(self) -> bool {
    self.left_assoc
  }
  pub const fn is_right_assoc(self) -> bool {
    self.right_assoc
  }
  pub const fn is_fully_assoc(self) -> bool {
    self.left_assoc && self.right_assoc
  }
}
