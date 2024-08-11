
use super::{LanguageMode, LanguageModeEngine, output_sep_by};
use crate::mode::display::unicode::{UnicodeAliasTable, common_unicode_aliases};
use crate::parsing::operator::{Operator, Precedence, OperatorTable};
use crate::parsing::operator::fixity::FixityType;
use crate::expr::Expr;
use crate::expr::number::{Number, ComplexNumber, Quaternion};
use crate::expr::atom::{Atom, write_escaped_str};
use crate::expr::basic_parser::ExprParser;
use crate::expr::vector::Vector;
use crate::expr::incomplete::{IncompleteObject, ObjectType};
use crate::util::cow_dyn::CowDyn;

use num::Zero;

#[derive(Clone, Debug, Default)]
pub struct BasicLanguageMode {
  known_operators: OperatorTable,
  unicode_table: UnicodeAliasTable,
  // Default is false. If true, the output should be readable by the
  // default parser. If false, some things may be pretty-printed (such
  // as incomplete objects).
  uses_reversible_output: bool,
}

impl BasicLanguageMode {
  pub fn new() -> Self {
    Self::default()
  }

  /// Replaces the [`UnicodeAliasTable`] associated with `self` and
  /// returns a modified version of `self`.
  pub fn with_unicode_table(mut self, table: UnicodeAliasTable) -> Self {
    self.unicode_table = table;
    self
  }

  pub fn from_operators(known_operators: OperatorTable) -> Self {
    Self {
      known_operators,
      unicode_table: UnicodeAliasTable::default(),
      uses_reversible_output: false,
    }
  }

  pub fn from_common_operators() -> Self {
    Self::from_operators(OperatorTable::common_operators())
      .with_unicode_table(common_unicode_aliases())
  }

  fn translate_to_unicode<'a>(&'a self, engine: &LanguageModeEngine, ascii_name: &'a str) -> &'a str {
    if !engine.language_settings().prefers_unicode_output || self.uses_reversible_output {
      // Technically, we could output Unicode in reversible mode,
      // since the parser supports it. But in the interests of showing
      // the user a more "canonical" version of the value, we disable
      // Unicode output unconditionally.
      return ascii_name;
    }
    self.unicode_table.get_unicode(ascii_name).unwrap_or(ascii_name)
  }

  fn fn_call_to_html(&self, engine: &LanguageModeEngine, out: &mut String, f: &str, args: &[Expr]) {
    let f = self.translate_to_unicode(engine, f);
    out.push_str(f);
    out.push('(');
    output_sep_by(out, args.iter(), ", ", |out, e| engine.write_to_html(out, e, Precedence::MIN));
    out.push(')');
  }

  // TODO Take InfixProperties directly, in addition to Operator?
  fn bin_infix_op_to_html(
    &self,
    engine: &LanguageModeEngine,
    out: &mut String,
    op: &Operator,
    left_arg: &Expr,
    right_arg: &Expr,
    prec: Precedence,
  ) {
    assert!(op.fixity().is_infix(), "Expected an infix operator: {:?}", op);
    let infix_props = op.fixity().as_infix().unwrap();
    let needs_parens = infix_props.precedence() < prec;
    if needs_parens {
      out.push('(');
    }
    engine.write_to_html(out, left_arg, infix_props.left_precedence());
    out.push(' ');
    // Special case: Infix multiplication can always be represented as
    // juxtaposition.
    if op.operator_name() != "*" {
      let operator_name = self.translate_to_unicode(engine, op.operator_name());
      out.push_str(operator_name);
      out.push(' ');
    }
    engine.write_to_html(out, right_arg, infix_props.right_precedence());
    if needs_parens {
      out.push(')');
    }
  }

  // TODO Take InfixProperties directly, in addition to Operator?
  fn variadic_infix_op_to_html(
    &self,
    engine: &LanguageModeEngine,
    out: &mut String,
    op: &Operator,
    args: &[Expr],
    prec: Precedence,
  ) {
    assert!(!args.is_empty());
    assert!(op.fixity().is_infix(), "Expected an infix operator: {:?}", op);
    let infix_props = op.fixity().as_infix().unwrap();
    let needs_parens = infix_props.precedence() < prec;
    if needs_parens {
      out.push('(');
    }
    let mut first = true;
    for arg in args {
      if !first {
        out.push(' ');
        // Special case: Infix multiplication can always be represented as
        // juxtaposition.
        if op.operator_name() != "*" {
          let operator_name = self.translate_to_unicode(engine, op.operator_name());
          out.push_str(operator_name);
          out.push(' ');
        }
      }
      first = false;
      engine.write_to_html(out, arg, infix_props.precedence());
    }
    if needs_parens {
      out.push(')');
    }
  }

  fn try_prefix_op_to_html(
    &self,
    engine: &LanguageModeEngine,
    out: &mut String,
    f: &str,
    args: &[Expr],
    prec: Precedence,
  ) -> bool {
    let Some(op) = self.known_operators.get_by_function_name(f, FixityType::Prefix) else {
      return false;
    };
    let Some(prefix_props) = op.fixity().as_prefix() else {
      return false;
    };
    if args.len() != 1 {
      return false;
    }
    let needs_parens = prefix_props.precedence() < prec;
    if needs_parens {
      out.push('(');
    }
    let operator_name = self.translate_to_unicode(engine, op.operator_name());
    out.push_str(operator_name);
    out.push(' ');
    engine.write_to_html(out, &args[0], prefix_props.precedence());
    if needs_parens {
      out.push(')');
    }
    true
  }

  fn try_postfix_op_to_html(&self, engine: &LanguageModeEngine, out: &mut String, f: &str, args: &[Expr], prec: Precedence) -> bool {
    let Some(op) = self.known_operators.get_by_function_name(f, FixityType::Postfix) else {
      return false;
    };
    let Some(postfix_props) = op.fixity().as_postfix() else {
      return false;
    };
    if args.len() != 1 {
      return false;
    }
    let needs_parens = postfix_props.precedence() < prec;
    if needs_parens {
      out.push('(');
    }
    engine.write_to_html(out, &args[0], postfix_props.precedence());
    out.push(' ');
    let operator_name = self.translate_to_unicode(engine, op.operator_name());
    out.push_str(operator_name);
    if needs_parens {
      out.push(')');
    }
    true
  }

  // Returns true if successful.
  fn try_infix_op_to_html(&self, engine: &LanguageModeEngine, out: &mut String, f: &str, args: &[Expr], prec: Precedence) -> bool {
    let Some(op) = self.known_operators.get_by_function_name(f, FixityType::Infix) else {
      return false;
    };
    let Some(infix_props) = op.fixity().as_infix() else {
      return false;
    };
    match args.len() {
      0 | 1 => {
        // Never write 0-ary or 1-ary functions as infix.
        false
      }
      2 => {
        self.bin_infix_op_to_html(engine, out, op, &args[0], &args[1], prec);
        true
      }
      _ => {
        if infix_props.associativity().is_fully_assoc() {
          self.variadic_infix_op_to_html(engine, out, op, args, prec);
          true
        } else {
          false
        }
      }
    }
  }

  fn number_needs_parens(&self, number: &Number, prec: Precedence) -> bool {
    let negation_precedence = self.known_operators
      .get_by_operator_name("-")
      .and_then(|op| op.fixity().as_prefix())
      .map(|op| op.precedence())
      .unwrap_or(Precedence::MIN);
    number < &Number::zero() && negation_precedence < prec
  }

  fn vector_to_html(&self, engine: &LanguageModeEngine, out: &mut String, elems: &[Expr]) {
    out.push('[');
    output_sep_by(out, elems.iter(), ", ", |out, e| engine.write_to_html(out, e, Precedence::MIN));
    out.push(']');
  }

  fn complex_to_html(&self, engine: &LanguageModeEngine, out: &mut String, args: &[Expr]) {
    assert_eq!(args.len(), 2, "Expecting slice of two Exprs, got {:?}", args);
    out.push('(');
    engine.write_to_html(out, &args[0], Precedence::MIN);
    out.push_str(", ");
    engine.write_to_html(out, &args[1], Precedence::MIN);
    out.push(')');
  }

  fn quat_to_html(&self, engine: &LanguageModeEngine, out: &mut String, args: &[Expr]) {
    assert_eq!(args.len(), 4, "Expecting slice of four Exprs, got {:?}", args);
    out.push('(');
    engine.write_to_html(out, &args[0], Precedence::MIN);
    out.push_str(", ");
    engine.write_to_html(out, &args[1], Precedence::MIN);
    out.push_str(", ");
    engine.write_to_html(out, &args[2], Precedence::MIN);
    out.push_str(", ");
    engine.write_to_html(out, &args[3], Precedence::MIN);
    out.push(')');
  }

  fn incomplete_object_to_html(&self, engine: &LanguageModeEngine, out: &mut String, args: &[Expr]) {
    assert_eq!(args.len(), 1, "Expecting slice of two Exprs, got {:?}", args);
    if let Expr::Atom(Atom::String(s)) = &args[0] {
      if let Ok(object_type) = ObjectType::parse(s) {
        let incomplete_object = IncompleteObject::new(object_type);
        out.push_str(&incomplete_object.to_string());
        return;
      }
    }
    self.fn_call_to_html(engine, out, IncompleteObject::FUNCTION_NAME, args);
  }
}

impl LanguageMode for BasicLanguageMode {
  fn write_to_html(&self, engine: &LanguageModeEngine, out: &mut String, expr: &Expr, prec: Precedence) {
    match expr {
      Expr::Atom(Atom::Number(n)) => {
        let needs_parens = self.number_needs_parens(n, prec);
        if needs_parens {
          out.push('(');
        }
        out.push_str(&n.to_string_radix(engine.language_settings().preferred_radix));
        if needs_parens {
          out.push(')');
        }
      }
      Expr::Atom(Atom::Var(v)) => {
        let var = self.translate_to_unicode(engine, v.as_str());
        out.push_str(var);
      }
      Expr::Atom(Atom::String(s)) => {
        write_escaped_str(out, s).unwrap(); // unwrap: impl Write for String doesn't fail.
      }
      Expr::Call(f, args) => {
        if !self.uses_reversible_output && f == IncompleteObject::FUNCTION_NAME && args.len() == 1 {
          self.incomplete_object_to_html(engine, out, args);
        } else if f == ComplexNumber::FUNCTION_NAME && args.len() == 2 {
          self.complex_to_html(engine, out, args);
        } else if f == Quaternion::FUNCTION_NAME && args.len() == 4 {
          self.quat_to_html(engine, out, args);
        } else if f == Vector::FUNCTION_NAME {
          self.vector_to_html(engine, out, args);
        } else {
          let as_op =
            self.try_infix_op_to_html(engine, out, f, args, prec) ||
            self.try_prefix_op_to_html(engine, out, f, args, prec) ||
            self.try_postfix_op_to_html(engine, out, f, args, prec);
          if !as_op {
            self.fn_call_to_html(engine, out, f, args);
          }
        }
      }
    }
  }

  fn to_trait_object(&self) -> &dyn LanguageMode {
    self
  }

  fn to_reversible_language_mode(&self) -> CowDyn<dyn LanguageMode> {
    if self.uses_reversible_output {
      CowDyn::Borrowed(self)
    } else {
      let mut language_mode = self.clone();
      language_mode.uses_reversible_output = true;
      CowDyn::Owned(Box::new(language_mode))
    }
  }

  fn parse(&self, text: &str) -> anyhow::Result<Expr> {
    let parser = ExprParser::new(&self.known_operators);
    let expr = parser.tokenize_and_parse(text)?;
    Ok(expr)
  }
}

#[cfg(test)]
mod tests {
  use super::*;
  use crate::mode::display::language::LanguageSettings;
  use crate::mode::display::unicode::{UnicodeAlias, UnicodeAliasTable};

  fn to_html<M>(mode: &M, expr: &Expr) -> String
  where M: LanguageMode + ?Sized {
    let settings = LanguageSettings::default();
    mode.to_html(expr, &settings)
  }

  fn to_html_no_unicode<M>(mode: &M, expr: &Expr) -> String
  where M: LanguageMode + ?Sized {
    let mut settings = LanguageSettings::default();
    settings.prefers_unicode_output = false;
    mode.to_html(expr, &settings)
  }

  fn sample_unicode_table() -> UnicodeAliasTable {
    UnicodeAliasTable::new(vec![
      UnicodeAlias::simple("A", "𝔸"),
      UnicodeAlias::simple("otimes", "⊗"),
    ]).unwrap()
  }

  #[test]
  fn test_atoms() {
    let mode = BasicLanguageMode::default();
    assert_eq!(to_html(&mode, &Expr::from(9)), "9");
    assert_eq!(to_html(&mode, &Expr::var("x").unwrap()), "x");
    assert_eq!(to_html(&mode, &Expr::from(r#"abc"def\"#)), r#""abc\"def\\""#);
  }

  #[test]
  fn test_complex_numbers() {
    let mode = BasicLanguageMode::default();
    assert_eq!(
      to_html(&mode, &Expr::from(ComplexNumber::new(2, -2))),
      "(2, -2)",
    );
    assert_eq!(
      to_html(&mode, &Expr::from(ComplexNumber::new(0, 2))),
      "(0, 2)",
    );
    assert_eq!(
      to_html(&mode, &Expr::from(ComplexNumber::new(-1, 0))),
      "(-1, 0)",
    );
  }

  #[test]
  fn test_simple_function_call() {
    let mode = BasicLanguageMode::default();
    let expr = Expr::call("foo", vec![Expr::from(9), Expr::from(8), Expr::from(7)]);
    assert_eq!(to_html(&mode, &expr), "foo(9, 8, 7)");
    let expr = Expr::call("foo", vec![Expr::from(9)]);
    assert_eq!(to_html(&mode, &expr), "foo(9)");
    let expr = Expr::call("foo", vec![]);
    assert_eq!(to_html(&mode, &expr), "foo()");
  }

  #[test]
  fn test_simple_function_call_with_unicode() {
    let mode = BasicLanguageMode::default()
      .with_unicode_table(sample_unicode_table());
    let expr = Expr::call("foo", vec![Expr::from(9), Expr::from(8), Expr::from(7)]);
    assert_eq!(to_html(&mode, &expr), "foo(9, 8, 7)");
    let expr = Expr::call("foo", vec![Expr::from(9)]);
    assert_eq!(to_html(&mode, &expr), "foo(9)");
    let expr = Expr::call("foo", vec![]);
    assert_eq!(to_html(&mode, &expr), "foo()");
    let expr = Expr::call("A", vec![]);
    assert_eq!(to_html(&mode, &expr), "𝔸()");
    let expr = Expr::call("A", vec![Expr::from(10), Expr::from(20)]);
    assert_eq!(to_html(&mode, &expr), "𝔸(10, 20)");
    let expr = Expr::call("A", vec![Expr::var("A").unwrap(), Expr::var("a").unwrap()]);
    assert_eq!(to_html(&mode, &expr), "𝔸(𝔸, a)");
  }

  #[test]
  fn test_simple_function_call_with_unicode_in_reversible_mode() {
    let mode = BasicLanguageMode::default()
      .with_unicode_table(sample_unicode_table());
    let mode = mode.to_reversible_language_mode();
    let expr = Expr::call("foo", vec![Expr::from(9), Expr::from(8), Expr::from(7)]);
    assert_eq!(to_html(mode.as_ref(), &expr), "foo(9, 8, 7)");
    let expr = Expr::call("foo", vec![Expr::from(9)]);
    assert_eq!(to_html(mode.as_ref(), &expr), "foo(9)");
    let expr = Expr::call("foo", vec![]);
    assert_eq!(to_html(mode.as_ref(), &expr), "foo()");
    let expr = Expr::call("A", vec![]);
    assert_eq!(to_html(mode.as_ref(), &expr), "A()");
    let expr = Expr::call("A", vec![Expr::from(10), Expr::from(20)]);
    assert_eq!(to_html(mode.as_ref(), &expr), "A(10, 20)");
    let expr = Expr::call("A", vec![Expr::var("A").unwrap(), Expr::var("a").unwrap()]);
    assert_eq!(to_html(mode.as_ref(), &expr), "A(A, a)");
  }

  #[test]
  fn test_simple_function_call_with_unicode_with_unicode_preferences_off() {
    let mode = BasicLanguageMode::default()
      .with_unicode_table(sample_unicode_table());
    let expr = Expr::call("foo", vec![Expr::from(9), Expr::from(8), Expr::from(7)]);
    assert_eq!(to_html_no_unicode(&mode, &expr), "foo(9, 8, 7)");
    let expr = Expr::call("foo", vec![Expr::from(9)]);
    assert_eq!(to_html_no_unicode(&mode, &expr), "foo(9)");
    let expr = Expr::call("foo", vec![]);
    assert_eq!(to_html_no_unicode(&mode, &expr), "foo()");
    let expr = Expr::call("A", vec![]);
    assert_eq!(to_html_no_unicode(&mode, &expr), "A()");
    let expr = Expr::call("A", vec![Expr::from(10), Expr::from(20)]);
    assert_eq!(to_html_no_unicode(&mode, &expr), "A(10, 20)");
    let expr = Expr::call("A", vec![Expr::var("A").unwrap(), Expr::var("a").unwrap()]);
    assert_eq!(to_html_no_unicode(&mode, &expr), "A(A, a)");
  }

  #[test]
  fn test_nested_function_call() {
    let mode = BasicLanguageMode::default();
    let expr = Expr::call(
      "foo",
      vec![
        Expr::from(10),
        Expr::call("bar", vec![]),
        Expr::call("baz", vec![Expr::from(3), Expr::from(-1)]),
      ],
    );
    assert_eq!(to_html(&mode, &expr), "foo(10, bar(), baz(3, -1))");
  }

  #[test]
  fn test_fully_assoc_op() {
    let mode = BasicLanguageMode::from_common_operators();
    let expr = Expr::call("+", vec![Expr::from(1), Expr::from(2), Expr::from(3)]);
    assert_eq!(to_html(&mode, &expr), "1 + 2 + 3");
  }

  #[test]
  fn test_fully_assoc_op_nested() {
    let mode = BasicLanguageMode::from_common_operators();
    let expr = Expr::call(
      "+",
      vec![
        Expr::call("+", vec![Expr::from(1), Expr::from(2)]),
        Expr::call("+", vec![Expr::from(3), Expr::from(4)]),
      ],
    );
    assert_eq!(to_html(&mode, &expr), "1 + 2 + 3 + 4");
  }

  #[test]
  fn test_assoc_ops_in_prec_order() {
    let mode = BasicLanguageMode::from_common_operators();
    let expr = Expr::call(
      "+",
      vec![
        Expr::call("*", vec![Expr::from(1), Expr::from(2)]),
        Expr::call("*", vec![Expr::from(3), Expr::from(4)]),
      ],
    );
    assert_eq!(to_html(&mode, &expr), "1 2 + 3 4");
  }

  #[test]
  fn test_assoc_ops_in_non_prec_order() {
    let mode = BasicLanguageMode::from_common_operators();
    let expr = Expr::call(
      "*",
      vec![
        Expr::call("+", vec![Expr::from(1), Expr::from(2)]),
        Expr::call("+", vec![Expr::from(3), Expr::from(4)]),
      ],
    );
    assert_eq!(to_html(&mode, &expr), "(1 + 2) (3 + 4)");
  }

  #[test]
  fn test_left_assoc_op() {
    let mode = BasicLanguageMode::from_common_operators();
    let expr = Expr::call(
      "-",
      vec![
        Expr::call("-", vec![Expr::from(1), Expr::from(2)]),
        Expr::call("-", vec![Expr::from(3), Expr::from(4)]),
      ],
    );
    assert_eq!(to_html(&mode, &expr), "1 - 2 - (3 - 4)");
  }

  #[test]
  fn test_right_assoc_op() {
    let mode = BasicLanguageMode::from_common_operators();
    let expr = Expr::call(
      "^",
      vec![
        Expr::call("^", vec![Expr::from(1), Expr::from(2)]),
        Expr::call("^", vec![Expr::from(3), Expr::from(4)]),
      ],
    );
    assert_eq!(to_html(&mode, &expr), "(1 ^ 2) ^ 3 ^ 4");
  }

  #[test]
  fn test_non_assoc_op() {
    let mode = BasicLanguageMode::from_common_operators();
    let expr = Expr::call(
      "%",
      vec![
        Expr::call("%", vec![Expr::from(1), Expr::from(2)]),
        Expr::call("%", vec![Expr::from(3), Expr::from(4)]),
      ],
    );
    assert_eq!(to_html(&mode, &expr), "(1 % 2) % (3 % 4)");
  }

  #[test]
  fn test_power_with_negative_base() {
    let mode = BasicLanguageMode::from_common_operators();
    let expr = Expr::call("^", vec![Expr::from(-1), Expr::from(2)]);
    assert_eq!(to_html(&mode, &expr), "(-1) ^ 2");
  }

  #[test]
  fn test_prefix_ops() {
    let mode = BasicLanguageMode::from_common_operators();
    let expr = Expr::call(
      "negate",
      vec![Expr::from(1)],
    );
    assert_eq!(to_html(&mode, &expr), "- 1");

    let expr = Expr::call(
      "identity",
      vec![Expr::from(1)],
    );
    assert_eq!(to_html(&mode, &expr), "+ 1");
  }

  #[test]
  fn test_vector() {
    let mode = BasicLanguageMode::from_common_operators();
    let expr = Expr::call(
      "vector",
      vec![Expr::from(1), Expr::from(2), Expr::from(3)],
    );
    assert_eq!(to_html(&mode, &expr), "[1, 2, 3]");
  }

  #[test]
  fn test_empty_vector() {
    let mode = BasicLanguageMode::from_common_operators();
    let expr = Expr::call(
      "vector",
      vec![],
    );
    assert_eq!(to_html(&mode, &expr), "[]");
  }

  #[test]
  fn test_singleton_vector() {
    let mode = BasicLanguageMode::from_common_operators();
    let expr = Expr::call(
      "vector",
      vec![Expr::from(-99)],
    );
    assert_eq!(to_html(&mode, &expr), "[-99]");
  }

  #[test]
  fn test_incomplete_object() {
    let mode = BasicLanguageMode::from_common_operators();
    let expr = Expr::call(
      "incomplete",
      vec![Expr::string("[")],
    );
    assert_eq!(to_html(&mode, &expr), "[ ...");
  }

  #[test]
  fn test_unicode_operator() {
    let mode = BasicLanguageMode::from_common_operators();
    let expr = Expr::call("<=", vec![Expr::from(100), Expr::from(200)]);
    assert_eq!(to_html(&mode, &expr), "100 ≤ 200");
    assert_eq!(to_html_no_unicode(&mode, &expr), "100 <= 200");
  }

  #[test]
  fn test_unicode_operator_in_reversible_mode() {
    let mode = BasicLanguageMode::from_common_operators();
    let mode = mode.to_reversible_language_mode();
    let expr = Expr::call("<=", vec![Expr::from(100), Expr::from(200)]);
    assert_eq!(to_html(mode.as_ref(), &expr), "100 <= 200");
    assert_eq!(to_html_no_unicode(mode.as_ref(), &expr), "100 <= 200");
  }

  #[test]
  fn test_incomplete_object_in_reversible_mode() {
    let mode = BasicLanguageMode::from_common_operators();
    let mode = mode.to_reversible_language_mode();
    let expr = Expr::call(
      "incomplete",
      vec![Expr::string("[")],
    );
    assert_eq!(mode.to_html(&expr, &LanguageSettings::default()), r#"incomplete("[")"#);
  }

  // TODO Common operators doesn't have any postfix ops right now,
  // test those when we get them
}
