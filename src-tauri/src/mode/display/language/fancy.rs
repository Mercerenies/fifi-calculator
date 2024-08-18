
use super::{LanguageMode, LanguageModeEngine};
use crate::parsing::operator::Precedence;
use crate::parsing::operator::table::{EXPONENT_PRECEDENCE, INTERVAL_PRECEDENCE,
                                      PREFIX_FUNCTION_CALL_PRECEDENCE};
use crate::mode::display::unicode::{UnicodeAliasTable, common_unicode_aliases};
use crate::util::cow_dyn::CowDyn;
use crate::expr::Expr;
use crate::expr::number::Number;
use crate::expr::vector::matrix::borrowed::BorrowedMatrix;
use crate::expr::var::Var;
use crate::expr::atom::Atom;
use crate::expr::interval::IntervalType;
use crate::util::brackets::{BracketConstruct, fancy_parens, fancy_square_brackets,
                            HtmlBrackets, HtmlBracketsType};

use once_cell::sync::Lazy;
use html_escape::encode_safe;
use num::Zero;

use std::collections::HashSet;

/// The fancy language mode, which uses HTML to render certain
/// function types using a more mathematical notation. In any case
/// that this language mode is not equipped to handle, it falls back
/// to its inner mode.
#[derive(Clone, Debug)]
pub struct FancyLanguageMode<L> {
  inner_mode: L,
  unicode_table: UnicodeAliasTable,
}

/// The set of function names eligible for prefix promotion. A
/// function call in this set will be promoted (in output) to a prefix
/// operator (with precedence [`PREFIX_FUNCTION_CALL_PRECEDENCE`]) if
/// the argument is simple enough. The phrase "simple enough" is
/// defined by the function [`can_prefix_promote_arg`]. For instance,
/// `sin(x)` will output as `sin x`, but more complex arguments like
/// `sin(x + y)` will be left alone.
///
/// Only unary functions are eligible for such promotion. Note that
/// the two-argument function `log` undergoes a similar promotion via
/// a special rule in [`FancyLanguageMode`].
pub static PREFIX_PROMOTION_FUNCTIONS: Lazy<HashSet<String>> = Lazy::new(|| {
  [
    "det", "trace",
    "ln",
    "sin", "cos", "tan", "sec", "csc", "cot",
    "sinh", "cosh", "tanh", "sech", "csch", "coth",
  ].into_iter().map(String::from).collect()
});

/// Returns true if the argument is simple enough for prefix
/// promotion. See [`PREFIX_PROMOTION_FUNCTIONS`] for more details on
/// this promotion rule.
///
/// Arguments which are simple enough for prefix promotion include:
///
/// * Nonnegative real numbers
///
/// * Variables
///
/// * String literals
pub fn can_prefix_promote_arg(arg: &Expr) -> bool {
  let Expr::Atom(atom) = arg else { return false; };
  match atom {
    Atom::Number(n) => *n >= Number::zero(),
    Atom::String(_) | Atom::Var(_) => true,
  }
}

impl<L: LanguageMode> FancyLanguageMode<L> {
  pub fn new(inner_mode: L) -> Self {
    Self {
      inner_mode,
      unicode_table: UnicodeAliasTable::default(),
    }
  }

  pub fn with_unicode_table(mut self, table: UnicodeAliasTable) -> Self {
    self.unicode_table = table;
    self
  }

  pub fn from_common_unicode(inner_mode: L) -> Self {
    Self::new(inner_mode)
      .with_unicode_table(common_unicode_aliases())
  }

  fn translate_to_unicode<'a>(&'a self, engine: &LanguageModeEngine, ascii_name: &'a str) -> &'a str {
    if !engine.language_settings().prefers_unicode_output {
      return ascii_name;
    }
    self.unicode_table.get_unicode(ascii_name).unwrap_or(ascii_name)
  }

  fn write_var(&self, engine: &LanguageModeEngine, out: &mut String, var: &Var) {
    let var_name = self.translate_to_unicode(engine, var.as_str());
    let var_name = encode_safe(var_name);
    out.push_str(r#"<span class="mathy-text">"#);
    out.push_str(var_name.as_ref());
    out.push_str("</span>");
  }

  fn write_exponent(&self, engine: &LanguageModeEngine, out: &mut String, args: &[Expr], prec: Precedence) {
    assert!(args.len() == 2);
    let [base, exp] = args else { unreachable!() };

    out.push_str("<span>");
    fancy_parens(true).write_bracketed_if_ok(out, prec > EXPONENT_PRECEDENCE, |out| {
      out.push_str("<span>");
      engine.write_to_html(out, base, EXPONENT_PRECEDENCE.incremented());
      out.push_str("</span>");
      out.push_str("<sup>");
      engine.write_to_html(out, exp, Precedence::MIN);
      out.push_str("</sup>");
    });
    out.push_str("</span>");
  }

  fn write_e_to_exponent(&self, engine: &LanguageModeEngine, out: &mut String, args: &[Expr], prec: Precedence) {
    assert!(args.len() == 1);
    let [exp] = args else { unreachable!() };

    out.push_str("<span>");
    fancy_parens(true).write_bracketed_if_ok(out, prec > EXPONENT_PRECEDENCE, |out| {
      out.push_str("<span>𝕖</span>");
      out.push_str("<sup>");
      engine.write_to_html(out, exp, Precedence::MIN);
      out.push_str("</sup>");
    });
    out.push_str("</span>");
  }

  fn write_abs_value_bars(
    &self,
    engine: &LanguageModeEngine,
    out: &mut String,
    inner_arg: &Expr,
    subscript_arg: Option<&Expr>,
  ) {
    let abs_value_bars = HtmlBrackets::new(HtmlBracketsType::VerticalBars);

    out.push_str("<span>");
    abs_value_bars.write_bracketed_if_ok(out, true, |out| {
      engine.write_to_html(out, inner_arg, Precedence::MIN);
    });
    if let Some(subscript_arg) = subscript_arg {
      out.push_str("<sub>");
      engine.write_to_html(out, subscript_arg, Precedence::MIN);
      out.push_str("</sub>");
    }
    out.push_str("</span>");
  }

  fn write_interval(&self, engine: &LanguageModeEngine, out: &mut String, interval_type: &str, args: &[Expr]) {
    assert!(args.len() == 2);
    let [left, right] = args else { unreachable!() };
    let interval_type = IntervalType::parse(interval_type).expect("Expected interval type in write_interval");
    let brackets = interval_type.html_brackets();
    let prec = INTERVAL_PRECEDENCE.incremented();

    out.push_str("<span>");
    brackets.write_bracketed_if_ok(out, true, |out| {
      engine.write_to_html(out, left, prec);
      out.push_str(" .. ");
      engine.write_to_html(out, right, prec);
    });
    out.push_str("</span>");
  }

  fn write_with_prefix_promotion(
    &self,
    engine: &LanguageModeEngine,
    out: &mut String,
    function: &str,
    arg: &Expr,
    prec: Precedence,
  ) {
    fancy_parens(true).write_bracketed_if_ok(out, prec > PREFIX_FUNCTION_CALL_PRECEDENCE, |out| {
      out.push_str(function);
      out.push(' ');
      engine.write_to_html(out, arg, PREFIX_FUNCTION_CALL_PRECEDENCE);
    });
  }

  fn write_logarithm_with_prefix_promotion(
    &self,
    engine: &LanguageModeEngine,
    out: &mut String,
    arg: &Expr,
    base: &Expr,
    prec: Precedence,
  ) {
    fancy_parens(true).write_bracketed_if_ok(out, prec > PREFIX_FUNCTION_CALL_PRECEDENCE, |out| {
      out.push_str("log<sub>");
      engine.write_to_html(out, base, Precedence::MIN);
      out.push_str("</sub> ");
      engine.write_to_html(out, arg, PREFIX_FUNCTION_CALL_PRECEDENCE);
    });
  }

  fn write_matrix(&self, engine: &LanguageModeEngine, out: &mut String, matrix: &BorrowedMatrix) {
    out.push_str("<span>");
    fancy_square_brackets(true).write_bracketed_if_ok(out, true, |out| {
      out.push_str(r#"<table class="matrix-table">"#);
      for row in matrix.rows() {
        out.push_str("<tr>");
        for elem in row {
          out.push_str("<td>");
          engine.write_to_html(out, elem, Precedence::MIN);
          out.push_str("</td>");
        }
        out.push_str("</tr>");
      }
      out.push_str("</table>");
    });
    out.push_str("</span>");
  }
}

impl<L: LanguageMode + Default> Default for FancyLanguageMode<L> {
  fn default() -> Self {
    Self::new(L::default())
  }
}

impl<L: LanguageMode> LanguageMode for FancyLanguageMode<L> {
  fn write_to_html(&self, engine: &LanguageModeEngine, out: &mut String, expr: &Expr, prec: Precedence) {
    if let Ok(matrix) = BorrowedMatrix::parse(expr) {
      self.write_matrix(engine, out, &matrix)
    } else {
      match expr {
        Expr::Atom(Atom::Number(_) | Atom::String(_)) => {
          self.inner_mode.write_to_html(engine, out, expr, prec)
        }
        Expr::Atom(Atom::Var(v)) => {
          self.write_var(engine, out, v)
        }
        Expr::Call(f, args) => {
          if f == "^" && args.len() == 2 {
            self.write_exponent(engine, out, args, prec)
          } else if f == "exp" && args.len() == 1 {
            self.write_e_to_exponent(engine, out, args, prec)
          } else if f == "abs" && args.len() == 1 {
            let [arg] = args.as_slice() else { unreachable!() };
            self.write_abs_value_bars(engine, out, arg, None)
          } else if f == "norm" && args.len() == 2 {
            let [arg, k] = args.as_slice() else { unreachable!() };
            self.write_abs_value_bars(engine, out, arg, Some(k))
          } else if IntervalType::is_interval_type(f) && args.len() == 2 {
            self.write_interval(engine, out, f, args)
          } else if PREFIX_PROMOTION_FUNCTIONS.contains(f) && args.len() == 1 && can_prefix_promote_arg(&args[0]) {
            self.write_with_prefix_promotion(engine, out, f, &args[0], prec)
          } else if f == "log" && args.len() == 2 && can_prefix_promote_arg(&args[0]) {
            self.write_logarithm_with_prefix_promotion(engine, out, &args[0], &args[1], prec)
          } else {
            self.inner_mode.write_to_html(engine, out, expr, prec)
          }
        }
      }
    }
  }

  fn to_trait_object(&self) -> &dyn LanguageMode {
    self
  }

  fn to_reversible_language_mode(&self) -> CowDyn<dyn LanguageMode> {
    self.inner_mode.to_reversible_language_mode()
  }

  fn parse(&self, text: &str) -> anyhow::Result<Expr> {
    self.inner_mode.parse(text)
  }
}

#[cfg(test)]
mod tests {
  use super::*;
  use crate::mode::display::language::test_utils::to_html;
  use crate::mode::display::language::basic::BasicLanguageMode;
  use crate::expr::number::ComplexNumber;

  fn sample_language_mode() -> FancyLanguageMode<BasicLanguageMode> {
    FancyLanguageMode::from_common_unicode(
      BasicLanguageMode::from_common_operators().with_fancy_parens(),
    )
  }

  #[test]
  fn test_can_prefix_promote_arg() {
    // Arguments simple enough to promote
    assert!(can_prefix_promote_arg(&Expr::from(19)));
    assert!(can_prefix_promote_arg(&Expr::from(0)));
    assert!(can_prefix_promote_arg(&Expr::from(0.0)));
    assert!(can_prefix_promote_arg(&Expr::from(0.2)));
    assert!(can_prefix_promote_arg(&Expr::string("xyz")));
    assert!(can_prefix_promote_arg(&Expr::string("")));
    assert!(can_prefix_promote_arg(&Expr::var("a").unwrap()));
    assert!(can_prefix_promote_arg(&Expr::var("X").unwrap()));

    // Arguments which cannot promote
    assert!(!can_prefix_promote_arg(&Expr::from(-1)));
    assert!(!can_prefix_promote_arg(&Expr::from(-0.2)));
    assert!(!can_prefix_promote_arg(&Expr::call("foo", vec![])));
    assert!(!can_prefix_promote_arg(&Expr::call("foo", vec![Expr::from(0)])));
    assert!(!can_prefix_promote_arg(&Expr::call("vector", vec![])));
  }

  #[test]
  fn test_atoms() {
    let mode = sample_language_mode();
    assert_eq!(to_html(&mode, &Expr::from(9)), "9");
    assert_eq!(to_html(&mode, &Expr::from(r#"abc"def\"#)), r#""abc\"def\\""#);
  }

  #[test]
  fn test_var_output() {
    let mode = sample_language_mode();
    assert_eq!(
      to_html(&mode, &Expr::var("x").unwrap()),
      r#"<span class="mathy-text">x</span>"#,
    );
  }

  #[test]
  fn test_complex_numbers() {
    let mode = sample_language_mode();
    assert_eq!(
      to_html(&mode, &Expr::from(ComplexNumber::new(2, -2))),
      r#"<span class="bracketed bracketed--parens">2, -2</span>"#,
    );
  }

  #[test]
  fn test_simple_function_call() {
    let mode = sample_language_mode();
    let expr = Expr::call("foo", vec![Expr::from(9), Expr::from(8), Expr::from(7)]);
    assert_eq!(
      to_html(&mode, &expr),
      r#"foo<span class="bracketed bracketed--parens">9, 8, 7</span>"#,
    );
  }

  #[test]
  fn test_unary_function_call() {
    let mode = sample_language_mode();
    let expr = Expr::call("foo", vec![Expr::from(9)]);
    assert_eq!(
      to_html(&mode, &expr),
      r#"foo<span class="bracketed bracketed--parens">9</span>"#,
    );
  }

  #[test]
  fn test_simple_function_call_in_reversible_mode() {
    let mode = sample_language_mode();
    let mode = mode.to_reversible_language_mode();
    let expr = Expr::call("foo", vec![Expr::from(9), Expr::from(8), Expr::from(7)]);
    assert_eq!(to_html(mode.as_ref(), &expr), "foo(9, 8, 7)");
  }

  #[test]
  fn test_e_to_power() {
    let mode = sample_language_mode();
    let expr = Expr::call("exp", vec![Expr::from(9)]);
    assert_eq!(
      to_html(&mode, &expr),
      r#"<span><span>𝕖</span><sup>9</sup></span>"#,
    );
  }

  #[test]
  fn test_power() {
    let mode = sample_language_mode();
    let expr = Expr::call("^", vec![Expr::from(2), Expr::from(10)]);
    assert_eq!(
      to_html(&mode, &expr),
      r#"<span><span>2</span><sup>10</sup></span>"#,
    );
  }

  #[test]
  fn test_power_nested_right() {
    let mode = sample_language_mode();
    let expr = Expr::call("^", vec![
      Expr::from(2),
      Expr::call("^", vec![Expr::from(3), Expr::from(10)]),
    ]);
    assert_eq!(
      to_html(&mode, &expr),
      r#"<span><span>2</span><sup><span><span>3</span><sup>10</sup></span></sup></span>"#,
    );
  }

  #[test]
  fn test_power_nested_left() {
    let mode = sample_language_mode();
    let expr = Expr::call("^", vec![
      Expr::call("^", vec![Expr::from(2), Expr::from(3)]),
      Expr::from(10),
    ]);
    assert_eq!(
      to_html(&mode, &expr),
      r#"<span><span><span><span class="bracketed bracketed--parens"><span>2</span><sup>3</sup></span></span></span><sup>10</sup></span>"#,
    );
  }

  #[test]
  fn test_power_of_e_nested_left() {
    let mode = sample_language_mode();
    let expr = Expr::call("^", vec![
      Expr::call("exp", vec![Expr::from(3)]),
      Expr::from(10),
    ]);
    assert_eq!(
      to_html(&mode, &expr),
      r#"<span><span><span><span class="bracketed bracketed--parens"><span>𝕖</span><sup>3</sup></span></span></span><sup>10</sup></span>"#,
    );
  }

  #[test]
  fn test_exp_with_wrong_arity() {
    let mode = sample_language_mode();
    let expr = Expr::call("exp", vec![Expr::from(2), Expr::from(3)]);
    assert_eq!(to_html(&mode, &expr), r#"exp<span class="bracketed bracketed--parens">2, 3</span>"#);
  }

  #[test]
  fn test_power_with_wrong_arity() {
    let mode = sample_language_mode();
    let expr = Expr::call("^", vec![Expr::from(2)]);
    assert_eq!(to_html(&mode, &expr), r#"^<span class="bracketed bracketed--parens">2</span>"#);
  }

  #[test]
  fn test_absolute_value() {
    let mode = sample_language_mode();
    let expr = Expr::call("abs", vec![Expr::from(-1)]);
    assert_eq!(
      to_html(&mode, &expr),
      r#"<span><span class="bracketed bracketed--vert">-1</span></span>"#,
    );
  }

  #[test]
  fn test_absolute_value_wrong_arity() {
    let mode = sample_language_mode();
    let expr = Expr::call("abs", vec![]);
    assert_eq!(
      to_html(&mode, &expr),
      r#"abs<span class="bracketed bracketed--parens"></span>"#,
    );
  }

  #[test]
  fn test_norm_wrong_arity() {
    let mode = sample_language_mode();
    let expr = Expr::call("norm", vec![Expr::from(-1)]);
    assert_eq!(
      to_html(&mode, &expr),
      r#"norm<span class="bracketed bracketed--parens">-1</span>"#,
    );
  }

  #[test]
  fn test_interval() {
    let mode = sample_language_mode();
    let expr = Expr::call("^..", vec![Expr::from(1), Expr::from(10)]);
    assert_eq!(
      to_html(&mode, &expr),
      r#"<span><span class="bracketed bracketed--parens-left bracketed--square-right">1 .. 10</span></span>"#,
    );
  }

  #[test]
  fn test_interval_wrong_arity() {
    let mode = sample_language_mode();
    let expr = Expr::call("^..", vec![Expr::from(1)]);
    assert_eq!(
      to_html(&mode, &expr),
      r#"^..<span class="bracketed bracketed--parens">1</span>"#,
    );
  }

  #[test]
  fn test_sin_with_simple_arg() {
    let mode = sample_language_mode();
    let expr = Expr::call("sin", vec![Expr::from(0.5)]);
    assert_eq!(
      to_html(&mode, &expr),
      r#"sin 0.5"#,
    );
  }

  #[test]
  fn test_sin_with_negative_arg() {
    let mode = sample_language_mode();
    let expr = Expr::call("sin", vec![Expr::from(-0.5)]);
    assert_eq!(
      to_html(&mode, &expr),
      r#"sin<span class="bracketed bracketed--parens">-0.5</span>"#,
    );
  }

  #[test]
  fn test_sin_with_wrong_arity() {
    let mode = sample_language_mode();
    let expr = Expr::call("sin", vec![]);
    assert_eq!(
      to_html(&mode, &expr),
      r#"sin<span class="bracketed bracketed--parens"></span>"#,
    );
  }

  #[test]
  fn test_logarithm() {
    let mode = sample_language_mode();
    let expr = Expr::call("log", vec![Expr::from(10), Expr::from(100)]);
    assert_eq!(
      to_html(&mode, &expr),
      r#"log<sub>100</sub> 10"#,
    );
  }
}
