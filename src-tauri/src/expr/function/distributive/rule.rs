
use crate::expr::Expr;
use crate::expr::predicates;
use crate::util::prism::ErrorWithPayload;
use super::eval::distribute_over;

use thiserror::Error;

/// A rule defining a place in an algebraic expression where the
/// distributive property can be applied.
pub struct DistributiveRule {
  outer_operator: String,
  inner_operator: String,
  side: Side,
  arg_rule: DistributiveArgRule,
}

/// A condition that must hold for an argument that is going to be
/// distributed over some other values.
pub struct DistributiveArgRule {
  body: Box<dyn Fn(&Expr) -> bool>,
}

#[derive(Debug, Clone, Error)]
#[error("Could not apply the distributive rule")]
pub struct DistributiveRuleNotApplicable {
  original_expr: Expr,
}

/// The side(s) on which the distributive property can be applied to a
/// given function.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Side {
  /// The distributive rule being defined can be applied to any
  /// arguments.
  Any,
  /// The distributive rule only applies to binary applications where
  /// the first argument is being distributed over the second.
  Left,
  /// The distributive rule only applies to binary applications where
  /// the second argument is being distributed over the first.
  Right,
}

impl DistributiveRule {
  pub fn new(
    outer_operator: impl Into<String>,
    inner_operator: impl Into<String>,
    side: Side,
  ) -> Self {
    Self {
      outer_operator: outer_operator.into(),
      inner_operator: inner_operator.into(),
      side,
      arg_rule: DistributiveArgRule::all_values_rule(),
    }
  }

  pub fn with_arg_rule(mut self, arg_rule: DistributiveArgRule) -> Self {
    self.arg_rule = arg_rule;
    self
  }

  /// Returns true if this distributive rule can be applied to the
  /// outermost call in the given expression, at the given index.
  ///
  /// A distributive rule can be applied to `expr` at `target_index`
  /// if and only if all of the following are true:
  ///
  /// * `expr` is a call expression whose outermost function call
  /// invokes the rule's outer operator;
  ///
  /// * `target_index` is among the [admissible argument
  /// positions](Side::admissible_args) for this distributive rule;
  ///
  /// * if [`Side::required_arity`] is not `None`, then the call's
  /// arity matches that value;
  ///
  /// * the `target_index`th argument is a call expression whose
  /// outermost function call is the rule's inner operator; and
  ///
  /// * all of the arguments that are not `target_index` satisfy the
  /// argument rule for this distributive rule.
  ///
  /// If this method returns true, then [`apply`](Self::apply) will
  /// return an `Ok` value for the given expression and the given
  /// target index.
  pub fn can_apply(&self, target_index: usize, expr: &Expr) -> bool {
    let Some((f, args)) = expr.as_call() else { return false; };
    if f != self.outer_operator {
      return false;
    }
    if let Some(required_arity) = self.side.required_arity() {
      if args.len() != required_arity {
        return false;
      }
    }
    if !self.side.admissible_args(args.len()).contains(&target_index) {
      return false;
    }
    args.iter().enumerate().all(|(i, arg)| {
      if i == target_index {
        // arg must be a call to the inner operator.
        matches!(arg, Expr::Call(inner_f, _) if inner_f == &self.inner_operator)
      } else {
        // arg must satisfy the argument rule.
        self.arg_rule.apply(arg)
      }
    })
  }

  /// Applies this distributive rule to the given index of the given
  /// expression, via [`distribute_over`]. If this rule does not
  /// apply, returns an error object containing the original
  /// expression argument, which can be recovered via
  /// [`recover_payload`](ErrorWithPayload::recover_payload).
  pub fn apply(&self, target_index: usize, expr: Expr) -> Result<Expr, DistributiveRuleNotApplicable> {
    if !self.can_apply(target_index, &expr) {
      return Err(DistributiveRuleNotApplicable { original_expr: expr });
    }
    // We check all of the conditions for distribute_over in
    // can_apply, so this will always be Ok.
    let expr = distribute_over(expr, target_index).expect("distribute_over failed after can_apply returned true");
    Ok(expr)
  }

  /// Applies this distributive rule to the first index for which it
  /// is applicable. If it is not applicable for any index, returns an
  /// error object containing the original expression argument.
  pub fn apply_first_match(&self, mut expr: Expr) -> Result<Expr, DistributiveRuleNotApplicable> {
    for index in self.side.admissible_args(expr.arity()) {
      match self.apply(index, expr) {
        Ok(expr) => { return Ok(expr); },
        Err(err) => { expr = err.recover_payload(); },
      }
    }
    Err(DistributiveRuleNotApplicable { original_expr: expr })
  }
}

impl DistributiveArgRule {
  pub fn new(body: impl Fn(&Expr) -> bool + 'static) -> Self {
    Self { body: Box::new(body) }
  }

  /// This condition holds for any real or complex number arguments.
  pub fn complex_number_rule() -> Self {
    Self::new(predicates::is_complex)
  }

  /// This condition is constantly true, i.e. it holds for all
  /// possible expressions.
  pub fn all_values_rule() -> Self {
    Self::new(|_| true)
  }

  pub fn apply(&self, arg: &Expr) -> bool {
    (self.body)(arg)
  }
}

impl Side {
  pub fn required_arity(&self) -> Option<usize> {
    match self {
      Self::Any => None,
      Self::Left => Some(2),
      Self::Right => Some(2),
    }
  }

  pub fn admissible_args(&self, expr_arity: usize) -> Vec<usize> {
    match self {
      Self::Any => (0..expr_arity).collect(),
      Self::Left => vec![1],
      Self::Right => vec![0],
    }
  }
}

impl ErrorWithPayload<Expr> for DistributiveRuleNotApplicable {
  fn recover_payload(self) -> Expr {
    self.original_expr
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  fn sample_always_rule() -> DistributiveRule {
    DistributiveRule::new("*", "+", Side::Any)
      .with_arg_rule(DistributiveArgRule::complex_number_rule())
  }

  fn sample_left_rule() -> DistributiveRule {
    DistributiveRule::new("*", "+", Side::Left)
      .with_arg_rule(DistributiveArgRule::complex_number_rule())
  }

  fn sample_right_rule() -> DistributiveRule {
    DistributiveRule::new("*", "+", Side::Right)
      .with_arg_rule(DistributiveArgRule::complex_number_rule())
  }

  #[test]
  fn test_apply_index_zero() {
    let expr = Expr::call("*", vec![
      Expr::call("+", vec![Expr::from(10), Expr::from(20)]),
      Expr::from(30),
      Expr::from(40),
    ]);
    let new_expr = sample_always_rule().apply(0, expr).unwrap();
    assert_eq!(new_expr, Expr::call("+", vec![
      Expr::call("*", vec![Expr::from(10), Expr::from(30), Expr::from(40)]),
      Expr::call("*", vec![Expr::from(20), Expr::from(30), Expr::from(40)]),
    ]));
  }

  #[test]
  fn test_apply_index_one() {
    let expr = Expr::call("*", vec![
      Expr::from(30),
      Expr::call("+", vec![Expr::from(10), Expr::from(20)]),
      Expr::from(40),
    ]);
    let new_expr = sample_always_rule().apply(1, expr).unwrap();
    assert_eq!(new_expr, Expr::call("+", vec![
      Expr::call("*", vec![Expr::from(30), Expr::from(10), Expr::from(40)]),
      Expr::call("*", vec![Expr::from(30), Expr::from(20), Expr::from(40)]),
    ]));
  }

  #[test]
  fn test_apply_index_out_of_bounds() {
    let expr = Expr::call("*", vec![
      Expr::from(30),
      Expr::call("+", vec![Expr::from(10), Expr::from(20)]),
      Expr::from(40),
    ]);
    sample_always_rule().apply(3, expr).unwrap_err();
  }

  #[test]
  fn test_apply_to_wrong_call_operator() {
    let expr = Expr::call("+", vec![
      Expr::from(30),
      Expr::call("*", vec![Expr::from(10), Expr::from(20)]),
      Expr::from(40),
    ]);
    sample_always_rule().apply(1, expr).unwrap_err();
    let expr = Expr::call("*", vec![
      Expr::from(30),
      Expr::call("*", vec![Expr::from(10), Expr::from(20)]),
      Expr::from(40),
    ]);
    sample_always_rule().apply(1, expr).unwrap_err();
  }

  #[test]
  fn test_apply_with_other_argument_invalid() {
    let expr = Expr::call("*", vec![
      Expr::from(30),
      Expr::call("+", vec![Expr::from(10), Expr::from(20)]),
      Expr::var("x").unwrap(), // This shouldn't distribute, because it's not a constant.
    ]);
    sample_always_rule().apply(0, expr.clone()).unwrap_err();
    sample_always_rule().apply(1, expr.clone()).unwrap_err();
    sample_always_rule().apply(2, expr).unwrap_err();
  }

  #[test]
  fn test_apply_index_to_atoms() {
    let expr = Expr::call("*", vec![Expr::from(10), Expr::from(20), Expr::from(30)]);
    sample_always_rule().apply(0, expr.clone()).unwrap_err();
    sample_always_rule().apply(1, expr.clone()).unwrap_err();
    sample_always_rule().apply(2, expr).unwrap_err();
    let expr = Expr::from(10);
    sample_always_rule().apply(0, expr).unwrap_err();
  }

  #[test]
  fn test_apply_left_side_rule() {
    let expr = Expr::call("*", vec![
      Expr::from(99),
      Expr::call("+", vec![Expr::from(10), Expr::from(20)]),
    ]);
    let new_expr = sample_left_rule().apply(1, expr).unwrap();
    assert_eq!(new_expr, Expr::call("+", vec![
      Expr::call("*", vec![Expr::from(99), Expr::from(10)]),
      Expr::call("*", vec![Expr::from(99), Expr::from(20)]),
    ]));
  }

  #[test]
  fn test_apply_left_side_rule_to_wrong_side() {
    let expr = Expr::call("*", vec![
      Expr::call("+", vec![Expr::from(10), Expr::from(20)]),
      Expr::from(99),
    ]);
    sample_left_rule().apply(0, expr).unwrap_err();
  }

  #[test]
  fn test_apply_right_side_rule() {
    let expr = Expr::call("*", vec![
      Expr::call("+", vec![Expr::from(10), Expr::from(20)]),
      Expr::from(99),
    ]);
    let new_expr = sample_right_rule().apply(0, expr).unwrap();
    assert_eq!(new_expr, Expr::call("+", vec![
      Expr::call("*", vec![Expr::from(10), Expr::from(99)]),
      Expr::call("*", vec![Expr::from(20), Expr::from(99)]),
    ]));
  }

  #[test]
  fn test_apply_right_side_rule_to_wrong_side() {
    let expr = Expr::call("*", vec![
      Expr::from(99),
      Expr::call("+", vec![Expr::from(10), Expr::from(20)]),
    ]);
    sample_right_rule().apply(1, expr).unwrap_err();
  }

  #[test]
  fn test_apply_right_side_rule_to_arity_one() {
    let expr = Expr::call("*", vec![
      Expr::call("+", vec![]),
    ]);
    sample_right_rule().apply(0, expr.clone()).unwrap_err();
    sample_right_rule().apply(1, expr).unwrap_err();
  }

  #[test]
  fn test_apply_right_side_rule_to_arity_zero() {
    let expr = Expr::call("*", vec![]);
    sample_right_rule().apply(0, expr).unwrap_err();
  }

  #[test]
  fn test_apply_left_side_rule_to_arity_three() {
    let expr = Expr::call("*", vec![
      Expr::from(99),
      Expr::call("+", vec![Expr::from(10), Expr::from(20)]),
      Expr::from(100),
    ]);
    sample_left_rule().apply(0, expr.clone()).unwrap_err();
    sample_left_rule().apply(1, expr.clone()).unwrap_err();
    sample_left_rule().apply(2, expr.clone()).unwrap_err();
    sample_left_rule().apply(3, expr).unwrap_err();
  }

  #[test]
  fn test_apply_right_side_rule_to_arity_three() {
    let expr = Expr::call("*", vec![
      Expr::from(99),
      Expr::call("+", vec![Expr::from(10), Expr::from(20)]),
      Expr::from(100),
    ]);
    sample_right_rule().apply(0, expr.clone()).unwrap_err();
    sample_right_rule().apply(1, expr.clone()).unwrap_err();
    sample_right_rule().apply(2, expr.clone()).unwrap_err();
    sample_right_rule().apply(3, expr).unwrap_err();
  }
}
