
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
use crate::util::brackets::{BracketConstruct, fancy_parens, fancy_square_brackets};

use html_escape::encode_safe;

use num::Zero;

/// The basic, and default, language mode. This language mode has
/// minimal support for sophisticated output or pretty-printing and is
/// designed to be mostly reversible.
#[derive(Clone, Debug, Default)]
pub struct BasicLanguageMode {
  known_operators: OperatorTable,
  unicode_table: UnicodeAliasTable,
  uses_fancy_parens: bool,
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

  /// Configures this language mode to use fancy parentheses. Rather
  /// than outputting ordinary `(` and `)` characters for parentheses,
  /// this language mode will output more sophisticated HTML which
  /// dynamically resizes the parentheses based on the content height.
  ///
  /// This flag is mutually exclusive with the reversible output flag.
  /// Setting the fancy parentheses un-sets the reversible output
  /// flag.
  pub fn with_fancy_parens(mut self) -> Self {
    self.uses_fancy_parens = true;
    self.uses_reversible_output = false;
    self
  }

  pub fn from_operators(known_operators: OperatorTable) -> Self {
    Self {
      known_operators,
      unicode_table: UnicodeAliasTable::default(),
      uses_reversible_output: false,
      uses_fancy_parens: false,
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
    out.push_str(encode_safe(f).as_ref());
    fancy_parens(self.uses_fancy_parens).write_bracketed_if_ok(out, true, |out| {
      output_sep_by(out, args.iter(), ", ", |out, e| engine.write_to_html(out, e, Precedence::MIN));
    });
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
    fancy_parens(self.uses_fancy_parens).write_bracketed_if_ok(out, prec > infix_props.precedence(), |out| {

      engine.write_to_html(out, left_arg, infix_props.left_precedence());
      out.push(' ');
      // Special case: Infix multiplication can always be represented as
      // juxtaposition.
      if op.operator_name() != "*" {
        let operator_name = self.translate_to_unicode(engine, op.operator_name());
        out.push_str(encode_safe(operator_name).as_ref());
        out.push(' ');
      }
      engine.write_to_html(out, right_arg, infix_props.right_precedence());
    });
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
    fancy_parens(self.uses_fancy_parens).write_bracketed_if_ok(out, prec > infix_props.precedence(), |out| {
      let mut first = true;
      for arg in args {
        if !first {
          out.push(' ');
          // Special case: Infix multiplication can always be represented as
          // juxtaposition.
          if op.operator_name() != "*" {
            let operator_name = self.translate_to_unicode(engine, op.operator_name());
            out.push_str(encode_safe(operator_name).as_ref());
            out.push(' ');
          }
        }
        first = false;
        engine.write_to_html(out, arg, infix_props.precedence());
      }
    });
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
    fancy_parens(self.uses_fancy_parens).write_bracketed_if_ok(out, prec > prefix_props.precedence(), |out| {
      let operator_name = self.translate_to_unicode(engine, op.operator_name());
      out.push_str(encode_safe(operator_name).as_ref());
      out.push(' ');
      engine.write_to_html(out, &args[0], prefix_props.precedence());
    });
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
    fancy_parens(self.uses_fancy_parens).write_bracketed_if_ok(out, prec > postfix_props.precedence(), |out| {
      engine.write_to_html(out, &args[0], postfix_props.precedence());
      out.push(' ');
      let operator_name = self.translate_to_unicode(engine, op.operator_name());
      out.push_str(encode_safe(operator_name).as_ref());
    });
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
    fancy_square_brackets(self.uses_fancy_parens).write_bracketed_if_ok(out, true, |out| {
      output_sep_by(out, elems.iter(), ", ", |out, e| engine.write_to_html(out, e, Precedence::MIN));
    });
  }

  fn complex_to_html(&self, engine: &LanguageModeEngine, out: &mut String, args: &[Expr]) {
    assert_eq!(args.len(), 2, "Expecting slice of two Exprs, got {:?}", args);
    fancy_parens(self.uses_fancy_parens).write_bracketed_if_ok(out, true, |out| {
      engine.write_to_html(out, &args[0], Precedence::MIN);
      out.push_str(", ");
      engine.write_to_html(out, &args[1], Precedence::MIN);
    });
  }

  fn quat_to_html(&self, engine: &LanguageModeEngine, out: &mut String, args: &[Expr]) {
    assert_eq!(args.len(), 4, "Expecting slice of four Exprs, got {:?}", args);
    fancy_parens(self.uses_fancy_parens).write_bracketed_if_ok(out, true, |out| {
      engine.write_to_html(out, &args[0], Precedence::MIN);
      out.push_str(", ");
      engine.write_to_html(out, &args[1], Precedence::MIN);
      out.push_str(", ");
      engine.write_to_html(out, &args[2], Precedence::MIN);
      out.push_str(", ");
      engine.write_to_html(out, &args[3], Precedence::MIN);
    });
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
        fancy_parens(self.uses_fancy_parens).write_bracketed_if_ok(out, self.number_needs_parens(n, prec), |out| {
          out.push_str(&n.to_string_radix(engine.language_settings().preferred_radix));
        });
      }
      Expr::Atom(Atom::Var(v)) => {
        let var = self.translate_to_unicode(engine, v.as_str());
        out.push_str(encode_safe(var).as_ref());
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
      language_mode.uses_fancy_parens = false;
      CowDyn::Owned(Box::new(language_mode))
    }
  }

  fn parse(&self, text: &str) -> anyhow::Result<Expr> {
    let parser = ExprParser::new(&self.known_operators);
    let expr = parser.tokenize_and_parse(text)?;
    Ok(expr)
  }

  fn language_mode_name(&self) -> String {
    String::from("Basic")
  }
}

#[cfg(test)]
mod tests {
  use super::*;
  use crate::mode::display::language::LanguageSettings;
  use crate::mode::display::language::test_utils::{to_html, to_html_no_unicode};
  use crate::mode::display::unicode::{UnicodeAlias, UnicodeAliasTable};

  fn sample_unicode_table() -> UnicodeAliasTable {
    UnicodeAliasTable::new(vec![
      UnicodeAlias::simple("A", "𝔸"),
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
  fn test_complex_numbers_with_fancy_parens() {
    let mode = BasicLanguageMode::default().with_fancy_parens();
    assert_eq!(
      to_html(&mode, &Expr::from(ComplexNumber::new(2, -2))),
      r#"<span class="bracketed bracketed--parens">2, -2</span>"#,
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
  fn test_simple_function_call_with_html_special_chars() {
    let mode = BasicLanguageMode::default();
    let expr = Expr::call("<span>", vec![Expr::from(9), Expr::from(8), Expr::from(7)]);
    assert_eq!(to_html(&mode, &expr), "&lt;span&gt;(9, 8, 7)");
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
  fn test_op_with_unicode() {
    let mode = BasicLanguageMode::from_common_operators();
    let expr = Expr::call("<", vec![Expr::from(1), Expr::from(2)]);
    assert_eq!(to_html(&mode, &expr), "1 &lt; 2");
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
  fn test_singleton_vector_with_fancy_parens_output() {
    let mode = BasicLanguageMode::from_common_operators().with_fancy_parens();
    let expr = Expr::call(
      "vector",
      vec![Expr::from(-99)],
    );
    assert_eq!(
      to_html(&mode, &expr),
      r#"<span class="bracketed bracketed--square">-99</span>"#,
    );
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
    assert_eq!(to_html_no_unicode(&mode, &expr), "100 &lt;= 200");
  }

  #[test]
  fn test_unicode_operator_in_reversible_mode() {
    let mode = BasicLanguageMode::from_common_operators();
    let mode = mode.to_reversible_language_mode();
    let expr = Expr::call("<=", vec![Expr::from(100), Expr::from(200)]);
    assert_eq!(to_html(mode.as_ref(), &expr), "100 &lt;= 200");
    assert_eq!(to_html_no_unicode(mode.as_ref(), &expr), "100 &lt;= 200");
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
