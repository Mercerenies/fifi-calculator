
use super::{LanguageMode, LanguageModeEngine, output_sep_by};
use crate::parsing::operator::{Operator, Precedence, OperatorTable};
use crate::parsing::operator::fixity::FixityType;
use crate::expr::Expr;
use crate::expr::number::{Number, ComplexNumber};
use crate::expr::atom::{Atom, write_escaped_str};
use crate::expr::basic_parser::ExprParser;
use crate::expr::vector::Vector;
use crate::util::cow_dyn::CowDyn;

use num::Zero;

#[derive(Clone, Debug, Default)]
pub struct BasicLanguageMode {
  known_operators: OperatorTable,
  // Default is false. If true, the output should be readable by the
  // default parser. If false, some things may be pretty-printed (such
  // as incomplete objects).
  uses_reversible_output: bool,
}

impl BasicLanguageMode {
  pub fn new() -> Self {
    Self::default()
  }

  pub fn from_operators(known_operators: OperatorTable) -> Self {
    Self {
      known_operators,
      uses_reversible_output: false,
    }
  }

  pub fn from_common_operators() -> Self {
    Self::from_operators(OperatorTable::common_operators())
  }

  fn fn_call_to_html(&self, engine: &LanguageModeEngine, out: &mut String, f: &str, args: &[Expr]) {
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
      out.push_str(op.operator_name());
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
          out.push_str(op.operator_name());
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
    out.push_str(op.operator_name());
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
    out.push_str(op.operator_name());
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
}

impl LanguageMode for BasicLanguageMode {
  fn write_to_html(&self, engine: &LanguageModeEngine, out: &mut String, expr: &Expr, prec: Precedence) {
    match expr {
      Expr::Atom(Atom::Number(n)) => {
        let needs_parens = self.number_needs_parens(n, prec);
        if needs_parens {
          out.push('(');
        }
        out.push_str(&n.to_string());
        if needs_parens {
          out.push(')');
        }
      }
      Expr::Atom(Atom::Var(v)) => {
        out.push_str(&v.to_string());
      }
      Expr::Atom(Atom::String(s)) => {
        write_escaped_str(out, s).unwrap(); // unwrap: impl Write for String doesn't fail.
      }
      Expr::Call(f, args) => {
        if f == ComplexNumber::FUNCTION_NAME && args.len() == 2 {
          self.complex_to_html(engine, out, args);
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
    CowDyn::Borrowed(self)
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

  #[test]
  fn test_atoms() {
    let mode = BasicLanguageMode::default();
    assert_eq!(mode.to_html(&Expr::from(9)), "9");
    assert_eq!(mode.to_html(&Expr::var("x").unwrap()), "x");
    assert_eq!(mode.to_html(&Expr::from(r#"abc"def\"#)), r#""abc\"def\\""#);
  }

  #[test]
  fn test_complex_numbers() {
    let mode = BasicLanguageMode::default();
    assert_eq!(
      mode.to_html(&Expr::from(ComplexNumber::new(2, -2))),
      "(2, -2)",
    );
    assert_eq!(
      mode.to_html(&Expr::from(ComplexNumber::new(0, 2))),
      "(0, 2)",
    );
    assert_eq!(
      mode.to_html(&Expr::from(ComplexNumber::new(-1, 0))),
      "(-1, 0)",
    );
  }

  #[test]
  fn test_simple_function_call() {
    let mode = BasicLanguageMode::default();
    let expr = Expr::call("foo", vec![Expr::from(9), Expr::from(8), Expr::from(7)]);
    assert_eq!(mode.to_html(&expr), "foo(9, 8, 7)");
    let expr = Expr::call("foo", vec![Expr::from(9)]);
    assert_eq!(mode.to_html(&expr), "foo(9)");
    let expr = Expr::call("foo", vec![]);
    assert_eq!(mode.to_html(&expr), "foo()");
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
    assert_eq!(mode.to_html(&expr), "foo(10, bar(), baz(3, -1))");
  }

  #[test]
  fn test_fully_assoc_op() {
    let mode = BasicLanguageMode::from_common_operators();
    let expr = Expr::call("+", vec![Expr::from(1), Expr::from(2), Expr::from(3)]);
    assert_eq!(mode.to_html(&expr), "1 + 2 + 3");
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
    assert_eq!(mode.to_html(&expr), "1 + 2 + 3 + 4");
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
    assert_eq!(mode.to_html(&expr), "1 2 + 3 4");
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
    assert_eq!(mode.to_html(&expr), "(1 + 2) (3 + 4)");
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
    assert_eq!(mode.to_html(&expr), "1 - 2 - (3 - 4)");
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
    assert_eq!(mode.to_html(&expr), "(1 ^ 2) ^ 3 ^ 4");
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
    assert_eq!(mode.to_html(&expr), "(1 % 2) % (3 % 4)");
  }

  #[test]
  fn test_power_with_negative_base() {
    let mode = BasicLanguageMode::from_common_operators();
    let expr = Expr::call("^", vec![Expr::from(-1), Expr::from(2)]);
    assert_eq!(mode.to_html(&expr), "(-1) ^ 2");
  }

  #[test]
  fn test_prefix_ops() {
    let mode = BasicLanguageMode::from_common_operators();
    let expr = Expr::call(
      "negate",
      vec![Expr::from(1)],
    );
    assert_eq!(mode.to_html(&expr), "- 1");

    let expr = Expr::call(
      "identity",
      vec![Expr::from(1)],
    );
    assert_eq!(mode.to_html(&expr), "+ 1");
  }

  #[test]
  fn test_vector() {
    let mode = BasicLanguageMode::from_common_operators();
    let expr = Expr::call(
      "vector",
      vec![Expr::from(1), Expr::from(2), Expr::from(3)],
    );
    assert_eq!(mode.to_html(&expr), "[1, 2, 3]");
  }

  #[test]
  fn test_empty_vector() {
    let mode = BasicLanguageMode::from_common_operators();
    let expr = Expr::call(
      "vector",
      vec![],
    );
    assert_eq!(mode.to_html(&expr), "[]");
  }

  #[test]
  fn test_singleton_vector() {
    let mode = BasicLanguageMode::from_common_operators();
    let expr = Expr::call(
      "vector",
      vec![Expr::from(-99)],
    );
    assert_eq!(mode.to_html(&expr), "[-99]");
  }

  // TODO Common operators doesn't have any postfix ops right now,
  // test those when we get them
}
