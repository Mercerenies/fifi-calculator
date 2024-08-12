
use super::base::{Simplifier, SimplifierContext};
use crate::mode::display::unicode::{UnicodeAliasTable, common_unicode_aliases};
use crate::expr::Expr;
use crate::expr::var::Var;
use crate::expr::atom::Atom;

/// A simplifier which recognizes Unicode names and simplifies them
/// down to their canonical ASCII equivalents.
#[derive(Debug)]
pub struct UnicodeSimplifier {
  table: UnicodeAliasTable,
}

impl UnicodeSimplifier {
  pub fn new(table: UnicodeAliasTable) -> Self {
    Self { table }
  }

  pub fn from_common_aliases() -> Self {
    Self::new(common_unicode_aliases())
  }

  fn lookup_ascii_name<'a>(&'a self, unicode_name: &'a str) -> &'a str {
    self.table.get_ascii(unicode_name).unwrap_or(unicode_name)
  }
}

impl Simplifier for UnicodeSimplifier {
  fn simplify_expr_part(&self, expr: Expr, _: &mut SimplifierContext) -> Expr {
    match expr {
      Expr::Atom(Atom::Var(var)) => {
        let canonical_name = self.lookup_ascii_name(var.as_str());
        let Some(new_var) = Var::new(canonical_name) else {
          eprintln!("UnicodeSimplifier: Got invalid variable name {canonical_name:?}");
          return Expr::Atom(Atom::Var(var));
        };
        Expr::Atom(Atom::Var(new_var))
      }
      Expr::Atom(Atom::String(_) | Atom::Number(_)) => expr,
      Expr::Call(name, args) => {
        Expr::call(self.lookup_ascii_name(name.as_str()), args)
      }
    }
  }
}

#[cfg(test)]
mod tests {
  use super::*;
  use crate::expr::simplifier::test_utils::run_simplifier;
  use crate::mode::display::unicode::UnicodeAlias;

  fn sample_simplifier() -> UnicodeSimplifier {
    let table = UnicodeAliasTable::new(vec![
      UnicodeAlias::simple("otimes", "⊗"),
      UnicodeAlias::simple("infty", "∞"),
      UnicodeAlias::new("less_or_equal", "≤", vec![String::from("⪯")]),
    ]).unwrap();
    UnicodeSimplifier::new(table)
  }

  #[test]
  fn test_unicode_simplifier() {
    let simplifier = sample_simplifier();

    let in_expr = Expr::call("≤", vec![
      Expr::from(10),
      Expr::call("⊗", vec![
        Expr::from(20),
        Expr::from(30),
        Expr::var("∞").unwrap(),
        Expr::var("infty").unwrap(),
      ]),
    ]);
    let (out_expr, errors) = run_simplifier(&simplifier, in_expr);
    assert!(errors.is_empty());
    assert_eq!(out_expr, Expr::call("less_or_equal", vec![
      Expr::from(10),
      Expr::call("otimes", vec![
        Expr::from(20),
        Expr::from(30),
        Expr::var("infty").unwrap(),
        Expr::var("infty").unwrap(),
      ]),
    ]));

    let in_expr = Expr::call("⪯", vec![
      Expr::from(10),
      Expr::call("otimes", vec![
        Expr::from(20),
        Expr::from(30),
        Expr::var("infty").unwrap(),
        Expr::var("∞").unwrap(),
      ]),
    ]);
    let (out_expr, errors) = run_simplifier(&simplifier, in_expr);
    assert!(errors.is_empty());
    assert_eq!(out_expr, Expr::call("less_or_equal", vec![
      Expr::from(10),
      Expr::call("otimes", vec![
        Expr::from(20),
        Expr::from(30),
        Expr::var("infty").unwrap(),
        Expr::var("infty").unwrap(),
      ]),
    ]));
  }
}
