
use super::LanguageMode;
use crate::error::Error;
use crate::parsing::operator::{Operator, Precedence, OperatorTable};
use crate::expr::Expr;
use crate::expr::atom::Atom;
use crate::expr::basic_parser::ExprParser;

#[derive(Clone, Debug, Default)]
pub struct BasicLanguageMode {
  known_operators: OperatorTable,
}

impl BasicLanguageMode {
  pub fn new() -> Self {
    Self::default()
  }

  pub fn from_operators(known_operators: OperatorTable) -> Self {
    Self {
      known_operators,
    }
  }

  pub fn from_common_operators() -> Self {
    Self::from_operators(OperatorTable::common_operators())
  }

  fn fn_call_to_html(&self, out: &mut String, f: &str, args: &[Expr]) {
    let mut first = true;
    out.push_str(f);
    out.push('(');
    args.iter().for_each(|e| {
      if !first {
        out.push_str(", ");
      }
      first = false;
      self.to_html_with_precedence(out, e, Precedence::MIN);
    });
    out.push(')');
  }

  // TODO Take InfixProperties directly, in addition to Operator?
  fn bin_infix_op_to_html(
    &self,
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
    self.to_html_with_precedence(out, left_arg, infix_props.left_precedence());
    out.push(' ');
    out.push_str(op.operator_name());
    out.push(' ');
    self.to_html_with_precedence(out, right_arg, infix_props.right_precedence());
    if needs_parens {
      out.push(')');
    }
  }

  // TODO Take InfixProperties directly, in addition to Operator?
  fn variadic_infix_op_to_html(&self, out: &mut String, op: &Operator, args: &[Expr], prec: Precedence) {
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
        out.push_str(op.operator_name());
        out.push(' ');
      }
      first = false;
      self.to_html_with_precedence(out, arg, infix_props.precedence());
    }
    if needs_parens {
      out.push(')');
    }
  }

  // Returns true if successful.
  fn try_infix_op_to_html(&self, out: &mut String, f: &str, args: &[Expr], prec: Precedence) -> bool {
    let Some(op) = self.known_operators.get_by_function_name(f) else {
      return false;
    };
    let Some(infix_props) = op.fixity().as_infix() else {
      return false;
    };
    match args.len() {
      0 => {
        // Never write 0-ary functions as infix.
        false
      }
      2 => {
        self.bin_infix_op_to_html(out, op, &args[0], &args[1], prec);
        true
      }
      _ => {
        if infix_props.associativity().is_fully_assoc() {
          self.variadic_infix_op_to_html(out, op, args, prec);
          true
        } else {
          false
        }
      }
    }
  }

  fn to_html_with_precedence(&self, out: &mut String, expr: &Expr, prec: Precedence) {
    match expr {
      Expr::Atom(Atom::Number(n)) => {
        out.push_str(&n.to_string());
      }
      Expr::Atom(Atom::Complex(z)) => {
        out.push_str(&z.to_string());
      }
      Expr::Call(f, args) => {
        let as_infix = self.try_infix_op_to_html(out, f, args, prec);
        if !as_infix {
          self.fn_call_to_html(out, f, args);
        }
      }
    }
  }
}

impl LanguageMode for BasicLanguageMode {
  fn write_to_html(&self, out: &mut String, expr: &Expr) {
    self.to_html_with_precedence(out, expr, Precedence::MIN)
  }

  fn parse(&self, text: &str) -> Result<Expr, Error> {
    let parser = ExprParser::new(&self.known_operators);
    parser.tokenize_and_parse(text).map_err(Error::custom_error)
  }
}

#[cfg(test)]
mod tests {
  use super::*;
  use crate::expr::number::{Number, ComplexNumber};

  #[test]
  fn test_atoms() {
    let mode = BasicLanguageMode::default();
    assert_eq!(mode.to_html(&Expr::from(9)), "9");
    assert_eq!(
      mode.to_html(&Expr::from(ComplexNumber::new(Number::from(2), Number::from(-2)))),
      "(2, -2)",
    );
    assert_eq!(
      mode.to_html(&Expr::from(ComplexNumber::new(Number::from(0), Number::from(2)))),
      "(0, 2)",
    );
    assert_eq!(
      mode.to_html(&Expr::from(ComplexNumber::new(Number::from(-1), Number::from(0)))),
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
    assert_eq!(mode.to_html(&expr), "1 * 2 + 3 * 4");
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
    assert_eq!(mode.to_html(&expr), "(1 + 2) * (3 + 4)");
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
  #[ignore = "Known bug, see Issue #16"]
  fn test_power_with_negative_base() {
    let mode = BasicLanguageMode::from_common_operators();
    let expr = Expr::call("^", vec![Expr::from(-1), Expr::from(2)]);
    assert_eq!(mode.to_html(&expr), "(-1) ^ 2");
  }
}
