
use super::rule::{DistributiveRule, DistributiveArgRule, Side};
use crate::expr::Expr;
use crate::util::prism::ErrorWithPayload;
use crate::expr::simplifier::{Simplifier, SimplifierContext};

use std::collections::HashMap;

/// A collection of distributive rules to be applied to expressions.
/// When there is a possibility of conflict, rules are always stored
/// in the order they were given, so rules added to the ruleset
/// earlier will have precedence over those added later.
#[derive(Default)]
pub struct DistributiveRuleset {
  map: HashMap<String, Vec<DistributiveRule>>,
}

/// Simplifier which attempts to run a distributive ruleset.
pub struct DistributiveRuleSimplifier {
  ruleset: DistributiveRuleset,
}

impl DistributiveRuleset {
  pub fn new() -> Self {
    Self::default()
  }

  /// Constructs a [`DistributiveRuleset`] consisting of all of the
  /// given rules.
  ///
  /// As per the precedence rules of `DistributiveRuleset`, rules
  /// earlier in the argument iterable take precedent over those later
  /// in the iterable.
  pub fn from_rules(rules: impl IntoIterator<Item = DistributiveRule>) -> Self {
    let mut map: HashMap<String, Vec<DistributiveRule>> = HashMap::new();
    for rule in rules {
      let name = rule.outer_operator().to_owned();
      map.entry(name).or_default().push(rule);
    }
    Self { map }
  }

  pub fn from_common_rules() -> Self {
    Self::from_rules([
      DistributiveRule::new("*", "+", Side::Any).with_arg_rule(DistributiveArgRule::complex_number_rule()),
      DistributiveRule::new("*", "-", Side::Any).with_arg_rule(DistributiveArgRule::complex_number_rule()),
      DistributiveRule::new("^", "*", Side::Right).with_arg_rule(DistributiveArgRule::complex_number_rule()),
      DistributiveRule::new("^", "/", Side::Right).with_arg_rule(DistributiveArgRule::complex_number_rule()),
      DistributiveRule::new("/", "+", Side::Right).with_arg_rule(DistributiveArgRule::complex_number_rule()),
      DistributiveRule::new("/", "-", Side::Right).with_arg_rule(DistributiveArgRule::complex_number_rule()),
    ])
  }

  /// Attempts to apply the first applicable distributive rule in this
  /// ruleset. Returns the original expression if no rule applies.
  pub fn apply(&self, mut expr: Expr) -> Expr {
    let Some(head) = expr.head() else { return expr; };
    let Some(rules) = self.map.get(head) else { return expr; };
    for rule in rules {
      match rule.apply_first_match(expr) {
        Ok(expr) => {
          return expr;
        }
        Err(err) => {
          expr = err.recover_payload();
        }
      }
    }
    expr
  }
}

impl DistributiveRuleSimplifier {
  pub fn new(ruleset: DistributiveRuleset) -> Self {
    Self { ruleset }
  }
}

impl Simplifier for DistributiveRuleSimplifier {
  fn simplify_expr_part(&self, expr: Expr, _ctx: &mut SimplifierContext) -> Expr {
    self.ruleset.apply(expr)
  }
}
